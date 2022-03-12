#![allow(unused)]
use futures_util::lock::MutexGuard;
use log::info;
use serde::{Deserialize, Serialize};
use tauri::async_runtime::spawn_blocking;

use crate::{client::RedditClient, Config, Post};
use std::{
    collections::HashMap,
    fs::{self, File},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

#[derive(Clone, Serialize, Deserialize)]
enum Status {
    Cloud,
    Local(PathBuf),
}

impl Default for Status {
    fn default() -> Self {
        Self::Cloud
    }
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct PostInfo {
    status: Status,
    selected: bool,
}

pub struct WallpaperManager {
    config: Arc<Config>,
    post_data: Mutex<HashMap<String, PostInfo>>,
    reddit_client: RedditClient,
    posts: Mutex<Vec<Arc<Post>>>,
    last_seen_wallpaper: Mutex<String>,
}

#[derive(Serialize, Deserialize)]
pub struct CachData {
    post_data: HashMap<String, PostInfo>,
    posts: Vec<Post>,
    last_seen_wallpaper: String,
}

impl From<&WallpaperManager> for CachData {
    fn from(wm: &WallpaperManager) -> Self {
        Self {
            post_data: (*wm.post_data.lock().unwrap()).clone(),
            posts: wm
                .posts
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
            posts: Mutex::new(vec![]),
            last_seen_wallpaper: Mutex::new("".to_string()),
        }
    }

    pub fn from_cache(config: Config) -> Self {
        let config = Arc::new(config);
        let reddit_client = RedditClient::new(config.clone());
        Self {
            config,
            post_data: Default::default(),
            reddit_client,
            posts: Mutex::new(vec![]),
            last_seen_wallpaper: Mutex::new("".to_string()),
        }
    }

    /// Fetch all wallpapers
    pub async fn fetch_all_wallpapers(&self) -> Vec<Post> {
        self.reddit_client
            .fetch_all_saved()
            .await
            .into_iter()
            .filter(|post| post.subreddit == "wallpaper")
            .collect::<Vec<_>>()
    }

    /// Get the cached wallpaper
    /// FIXME: don't clone
    pub async fn get_cached_wallpapers(&self) -> Vec<Arc<Post>> {
        self.posts.lock().unwrap().clone()
    }

    /// Set a wallpaper as system-wallpaper
    pub async fn set_wallpaper(&self, name: &str) {
        let post = self
            .get_post(name)
            .unwrap_or_else(|| panic!("no post with name {} exists", name));
        let status = self
            .post_data
            .lock()
            .unwrap()
            .get(name)
            .expect("no post_data for this post")
            .status
            .clone();

        match status {
            Status::Cloud => {
                let save_path = self
                    .reddit_client
                    .download_post_image(self.config.path.clone(), post)
                    .await
                    .await
                    .unwrap();
                info!("{:?}", save_path.to_str());
                wallpaper::set_from_path((&save_path.canonicalize().unwrap()).to_str().unwrap());
                let mut post_data = self.post_data.lock().unwrap();
                let single_post_data = post_data.get_mut(name).unwrap();
                single_post_data.status = Status::Local(save_path);
            }
            Status::Local(path) => {
                wallpaper::set_from_path(path.to_str().unwrap());
            }
        }
    }

    fn get_post(&self, name: &str) -> Option<Arc<Post>> {
        self.posts
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
            .map(Arc::from)
            .collect::<Vec<_>>();

        // create info for all the posts
        posts.iter().for_each(|post| {
            self.post_data
                .lock()
                .unwrap()
                .insert(post.name.clone(), Default::default());
        });
        self.posts.lock().unwrap().extend(posts);
        info!("we now have: {:?} posts", self.posts.lock().unwrap().len());
        *self.last_seen_wallpaper.lock().unwrap() = new_last_seen;
    }

    /// Save all important data to the disc
    pub fn save(&self) {
        let file = File::create("cache.json").unwrap();
        serde_json::to_writer(file, &CachData::from(self));
    }
}
