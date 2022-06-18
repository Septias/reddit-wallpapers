#![allow(unused)]
use futures_util::{lock::MutexGuard, future::join_all};
use image::io::Reader;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use tauri::async_runtime::spawn_blocking;
use tokio::fs::create_dir;

use crate::{client::RedditClient, Config, Post, VALID_EXTENSION};
use std::{
    collections::HashMap,
    fs::{self, read_to_string, File},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct PostInfo {
    selected: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Wallpaper {
    pub subreddit: String,
    pub title: String,
    pub url: String,
    pub name: String,
    pub file_name: String,
}

pub struct WallpaperManager {
    config: Arc<Config>,
    post_data: Mutex<HashMap<String, PostInfo>>,
    reddit_client: RedditClient,
    wallpapers: Mutex<Vec<Arc<Wallpaper>>>,
    last_seen_wallpaper: Mutex<String>,
}

#[derive(Serialize, Deserialize)]
pub struct CachData {
    post_data: HashMap<String, PostInfo>,
    posts: Vec<Wallpaper>,
    last_seen_wallpaper: String,
}

impl From<&WallpaperManager> for CachData {
    fn from(wm: &WallpaperManager) -> Self {
        Self {
            post_data: (*wm.post_data.lock().unwrap()).clone(),
            posts: wm
                .wallpapers
                .lock()
                .unwrap()
                .iter()
                .map(|post| (**post).clone())
                .collect::<Vec<_>>(),
            last_seen_wallpaper: wm.last_seen_wallpaper.lock().unwrap().clone(),
        }
    }
}

/// Define which data should be cached
impl WallpaperManager {
    pub fn new(config: Config) -> Self {
        let config = Arc::new(config);
        let reddit_client = RedditClient::new(config.clone());
        Self {
            config,
            post_data: Default::default(),
            reddit_client,
            wallpapers: Mutex::new(vec![]),
            last_seen_wallpaper: Mutex::new("".to_string()),
        }
    }

    pub fn from_cache(config: Config) -> Self {
        let config = Arc::new(config);
        let reddit_client = RedditClient::new(config.clone());
        let cach_data: CachData =
            serde_json::from_str(&read_to_string("./cache.json").expect("no config file found"))
                .unwrap();
        Self {
            config,
            post_data: Mutex::new(cach_data.post_data),
            reddit_client,
            wallpapers: Mutex::new(
                cach_data
                    .posts
                    .into_iter()
                    .map(Arc::new)
                    .collect::<Vec<_>>(),
            ),
            last_seen_wallpaper: Mutex::new(cach_data.last_seen_wallpaper),
        }
    }

    /// Fetch all wallpapers
    pub async fn fetch_all_wallpapers(&self) -> Vec<Post> {
        self.reddit_client
            .fetch_all_saved_posts()
            .await
            .into_iter()
            .filter(|post| post.subreddit == "wallpaper")
            .collect::<Vec<_>>()
    }

    /// Get the cached wallpaper
    /// FIXME: don't clone
    pub async fn get_cached_wallpapers(&self) -> Vec<Arc<Wallpaper>> {
        self.wallpapers.lock().unwrap().clone()
    }

    /// Set a wallpaper as system-wallpaper
    pub async fn set_wallpaper(&self, name: &str) {
        let wallpaper = self
            .get_wallpaper(name)
            .unwrap_or_else(|| panic!("no post with name {} exists", name));

        let path = self.config.path.join(&wallpaper.file_name);
        let path = path.to_str().unwrap();
        info!("setting wallpaper: {:?}", path);
        wallpaper::set_from_path(path).unwrap();
    }

    fn get_wallpaper(&self, name: &str) -> Option<Arc<Wallpaper>> {
        self.wallpapers
            .lock()
            .unwrap()
            .iter()
            .find(|post| post.name == name)
            .cloned()
    }

    /// Fetch all new wallpapers from reddit api
    pub async fn fetch_recent_wallpapers(&self) {
        let mut new_last_seen = self.last_seen_wallpaper.lock().unwrap().clone();

        let posts = self
            .reddit_client
            .fetch_saved_until(&mut new_last_seen)
            .await
            .into_iter()
            .filter(|post| post.subreddit == "wallpaper")
            .filter(|post| {
                let valid = VALID_EXTENSION.contains(&post.url.split('.').last().unwrap());
                if !valid {
                    warn!(
                        "not adding resource {}, because it has no valid picture-ending",
                        post.url
                    );
                }
                valid
            })
            .map(Arc::from)
            .collect::<Vec<_>>();

        let paths = self.reddit_client.downloader_post_images(&posts).await;

        let thumbnails_path = self.wallpaper_path().join("thumbnails");
        if !thumbnails_path.exists() {
            create_dir(&thumbnails_path).await.unwrap();
        }
        let mut futures = vec![];
        for file_name in paths.values() {
            let file_name = file_name.to_owned();
            let thumbnails_path = thumbnails_path.clone();
            let file_path = self.wallpaper_path().join(&file_name);
            let future = spawn_blocking(move || {
                let single_path = thumbnails_path.join(&file_name);
                if (single_path.exists()) {
                    return;
                }
                let image = Reader::open(&file_path)
                    .unwrap()
                    .decode()
                    .unwrap();
                let factor = image.height() as f32 / image.width() as f32;
                let thumbnail = image.thumbnail(300, (300. * factor) as u32);
                thumbnail.save(&single_path).unwrap();
                info!("generated thumbnail {:?}", &single_path);
            });
            futures.push(future);
        }
        for future in futures {
            future.await.unwrap();
        }

        // create info for all the posts
        posts.iter().for_each(|post| {
            self.post_data
                .lock()
                .unwrap()
                .insert(post.name.clone(), Default::default());
        });

        let wallpapers = posts.into_iter().map(|post| {
            let post = Arc::try_unwrap(post).unwrap();
            Arc::new(Wallpaper {
                subreddit: post.subreddit,
                title: post.title,
                url: post.url,
                file_name: paths.get(&post.name).unwrap().clone(),
                name: post.name,
            })
        });

        self.wallpapers.lock().unwrap().extend(wallpapers);
        info!("new image count: {}", self.wallpapers.lock().unwrap().len());
        *self.last_seen_wallpaper.lock().unwrap() = new_last_seen;
    }

    /// Save all important data to the disc
    pub fn save(&self) {
        let file = File::create("cache.json").unwrap();
        serde_json::to_writer(file, &CachData::from(self));
    }

    pub fn wallpaper_path(&self) -> &Path {
        &self.config.path.as_path()
    }
}
