#![allow(unused)]
use futures_util::{future::join_all, lock::MutexGuard};
use image::io::{Reader};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use tauri::async_runtime::spawn_blocking;
use thiserror::Error;
use tokio::fs::create_dir;

use crate::{
    client::{ClientError, RedditClient},
    Config, Post, VALID_EXTENSION, WallpaperError,
};
use std::{
    collections::HashMap,
    fs::{self, read_to_string, File},
    path::{Path, PathBuf},
    sync::{Arc, Mutex}, any::Any, io,
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
    pub config: Mutex<Config>,
    post_data: Mutex<HashMap<String, PostInfo>>,
    reddit_client: Mutex<Option<RedditClient>>,
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

impl WallpaperManager {
    pub async fn new() -> Self {
        let config = read_to_string("./wallpapers.toml")
            .ok()
            .map(|content| toml::from_str(&content).unwrap())
            .unwrap_or_default();
        let reddit_client = RedditClient::new(&config).await;
        if let Err(e) = &reddit_client {
            warn!("{e}")
        }
        Self {
            reddit_client: Mutex::new(reddit_client.ok()),
            config: Mutex::new(config),
            post_data: Default::default(),
            wallpapers: Mutex::new(vec![]),
            last_seen_wallpaper: Mutex::new("".to_string()),
        }
    }

    pub async fn from_cache() -> Self {
        let config = read_to_string("./wallpapers.toml")
            .ok()
            .map(|content| toml::from_str(&content).unwrap())
            .unwrap_or_default();
        let cach_data: CachData =
            serde_json::from_str(&read_to_string("./cache.json").expect("no config file found"))
                .unwrap();
        let reddit_client = RedditClient::new(&config).await;
        if let Err(e) = &reddit_client {
            warn!("{e}")
        }
        Self {
            reddit_client: Mutex::new(reddit_client.ok()),
            config: Mutex::new(config),
            post_data: Mutex::new(cach_data.post_data),
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
    pub async fn fetch_all_wallpapers(&self) -> Result<Vec<Post>, ClientError> {
        let client = self.get_client()?;
        let wallpapers = client
            .fetch_all_saved_posts()
            .await
            .into_iter()
            .filter(|post| post.subreddit == "wallpaper")
            .collect::<Vec<_>>();
        self.put_client(client);
        Ok(wallpapers)
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
        let config = self.config.lock().unwrap();
        let path = config.path.join(&wallpaper.file_name);
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

    fn get_client(&self) -> Result<RedditClient, ClientError> {
        let rc = self.reddit_client.lock().unwrap().take();
        rc.ok_or(ClientError::BadCredetials)
    }

    fn put_client(&self, client: RedditClient) {
        *self.reddit_client.lock().unwrap() = Some(client)
    }

    /// Fetch all new wallpapers from reddit app
    pub async fn fetch_recent_wallpapers(&self) -> Result<(), ClientError> {
        let mut new_last_seen = self.last_seen_wallpaper.lock().unwrap().clone();
        let client = self.get_client()?;
        // request all new post
        let posts = client.fetch_saved_until(&mut new_last_seen).await;

        // filter posts
        let posts = {
            let wallpapers = self.wallpapers.lock().unwrap();
            posts
                .into_iter()
                .filter(|post| {
                    let wallpapers_subreddit = post.subreddit == "wallpaper";
                    let already_present =
                        wallpapers.iter().find(|wp| wp.name == post.name).is_some();
                    let valid_extension =
                        VALID_EXTENSION.contains(&post.url.split('.').last().unwrap());

                    if wallpapers_subreddit && !valid_extension {
                        warn!(
                            "not adding resource {}, because it has no valid picture-ending",
                            post.url
                        );
                    }

                    valid_extension && wallpapers_subreddit && !already_present
                })
                .map(Arc::from)
                .collect::<Vec<_>>()
        };

        // download all background images
        let paths = client.downloader_post_images(&posts).await;
        self.create_thumbnails(&paths).await;

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
        self.put_client(client);
        Ok(())
    }

    async fn create_thumbnails(&self, paths: &HashMap<String, String>) {
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
                let image = Reader::open(&file_path).unwrap().decode().unwrap();
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
    }

    pub fn wallpaper_path(&self) -> PathBuf {
        self.config.lock().unwrap().path.clone()
    }

    pub async fn set_config(&self, config: Config) -> Result<(), WallpaperError> {
        let client = RedditClient::new(&config).await?;
        *self.reddit_client.lock().unwrap() = Some(client);
        fs::try_exists(&config.path).map_err(|_| WallpaperError::PathDoesNotExist)?;
        fs::write("wallpapers.toml", toml::to_string(&config).unwrap());
        *self.config.lock().unwrap() = config;
        info!("saving new user data");
        Ok(())
    }

    pub fn is_configured(&self) -> bool{
        self.reddit_client.lock().unwrap().is_some()
    }

    /// Save all important data to the disc
    pub fn save(&self) {
        let file = File::create("cache.json").unwrap();
        serde_json::to_writer(file, &CachData::from(self));
    }
}