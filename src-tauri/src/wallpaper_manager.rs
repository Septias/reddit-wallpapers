#![allow(unused)]
use futures_util::lock::MutexGuard;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use tauri::async_runtime::spawn_blocking;

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
    path: PathBuf,
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

        info!("setting wallpaper: {:?}", wallpaper.path.to_str());
        wallpaper::set_from_path(wallpaper.path.to_str().unwrap()).unwrap();
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
                path: paths.get(&post.name).unwrap().clone(),
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
}
