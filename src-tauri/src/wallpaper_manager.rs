use image::io::Reader;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use tauri::{
    api::{
        file::read_string,
        path::{cache_dir, config_dir},
    },
    async_runtime::spawn_blocking,
};
use tokio::fs::create_dir;

use crate::{
    client::{ClientError, RedditClient},
    Config, Post, WallpaperError, VALID_EXTENSION,
};
use std::{
    collections::HashMap,
    fs::{self, create_dir_all},
    path::PathBuf,
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
    pub config: Mutex<Vec<Config>>,
    post_data: Mutex<HashMap<String, PostInfo>>,
    reddit_clients: Mutex<Vec<RedditClient>>, // Changed from single RedditClient
    wallpapers: Mutex<Vec<Arc<Wallpaper>>>,
    last_seen_wallpaper: Mutex<HashMap<String, String>>, // Changed to track last seen per account
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
    /// Create a new WallpaperManager
    /// Tries to load cache from filesystem
    /// Tries to load config from filesystem
    pub async fn new() -> Self {
        // load config
        let config = Self::load_configs().unwrap_or_default();

        // create clients using config
        let mut reddit_clients = Vec::new();
        for conf in &config {
            match RedditClient::new(conf).await {
                Ok(client) => reddit_clients.push(client),
                Err(e) => warn!("{e}"),
            }
        }

        // load post_data and wallpapers
        let (post_data, wallpapers, last_seen_wallpaper) = Self::load_cache().unwrap_or_default();
        Self {
            reddit_clients: Mutex::new(reddit_clients),
            config: Mutex::new(config),
            post_data,
            wallpapers,
            last_seen_wallpaper, // Now a map
        }
    }

    fn config_path() -> Option<PathBuf> {
        config_dir().map(|mut path| {
            path.push("reddit-wallpapers/wallpapers.toml");
            path
        })
    }

    /// Tries to read config from filesystem
    fn load_configs() -> Option<Vec<Config>> {
        if let Some(path) = Self::config_path() {
            let data = read_string(path)
                .ok()
                .map(|content| toml::from_str::<Vec<Config>>(&content).unwrap());
            info!("successfully loaded config");
            data
        } else {
            warn!("can't create config path");
            None
        }
    }

    fn save_configs(config: &[Config]) -> anyhow::Result<()> {
        if let Some(path) = Self::config_path() {
            info!("saving config at {path:?}");
            let parent = path.parent().unwrap();
            if !parent.is_dir() {
                create_dir_all(parent)?;
            }
            fs::write(path, toml::to_string(&config).unwrap()).unwrap();
        } else {
            warn!("can't create config path");
        }
        Ok(())
    }

    fn cache_path() -> Option<PathBuf> {
        cache_dir().map(|mut path| {
            path.push("reddit-wallpapers/cache.json");
            path
        })
    }

    fn load_cache() -> Option<(
        Mutex<HashMap<String, PostInfo>>,
        Mutex<Vec<Arc<Wallpaper>>>,
        Mutex<String>,
    )> {
        if let Some(path) = Self::cache_path() {
            let data = read_string(path)
                .ok()
                .and_then(|content| serde_json::from_str::<CachData>(&content).ok())
                .map(|a| {
                    (
                        Mutex::new(a.post_data),
                        Mutex::new(a.posts.into_iter().map(Arc::new).collect::<Vec<_>>()),
                        Mutex::new(a.last_seen_wallpaper),
                    )
                });
            info!("successfully loaded cache");
            data
        } else {
            warn!("can't create cache path");
            None
        }
    }

    /// Save cache to disk
    pub fn save_cache(&self) -> anyhow::Result<()> {
        if let Some(path) = Self::cache_path() {
            info!("saving cache at {path:?}");
            let parent = path.parent().unwrap();
            if !parent.exists() {
                create_dir_all(parent)?;
            }
            let data = serde_json::to_string(&CachData::from(self)).unwrap();
            fs::write(path, data)?;
        } else {
            warn!("can't create config path");
        }
        Ok(())
    }

    /// Fetch all wallpapers from all Reddit clients
    pub async fn fetch_all_wallpapers(&self) -> Result<Vec<Post>, ClientError> {
        let clients = self.get_clients()?;
        let mut all_wallpapers = Vec::new();

        for client in clients.iter() {
            let wallpapers = client
                .fetch_all_saved_posts()
                .await
                .into_iter()
                .filter(|post| post.subreddit == "wallpaper")
                .collect::<Vec<_>>();
            all_wallpapers.extend(wallpapers);
        }

        self.put_clients(clients);
        Ok(all_wallpapers)
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
            .unwrap_or_else(|| panic!("no wallpaper with name {} exists", name));
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

    fn get_clients(&self) -> Result<Vec<RedditClient>, ClientError> {
        let clients = self.reddit_clients.lock().unwrap();
        if clients.is_empty() {
            Err(ClientError::BadCredetials)
        } else {
            Ok(clients.clone())
        }
    }

    fn put_clients(&self, clients: Vec<RedditClient>) {
        *self.reddit_clients.lock().unwrap() = clients;
    }

    /// Fetch recent wallpapers from all Reddit clients
    pub async fn fetch_recent_wallpapers(&self) -> Result<(), ClientError> {
        let clients = self.get_clients()?;
        info!("started fetching wallpapers from all accounts");

        for client in clients.iter_mut() {
            let account = &client.username;
            let last_seen = self
                .last_seen_wallpaper
                .lock()
                .unwrap()
                .get(account)
                .cloned()
                .unwrap_or_default();
            let (posts, new_last_seen) = client.fetch_saved_until(&last_seen).await?;
            self.last_seen_wallpaper
                .lock()
                .unwrap()
                .insert(account.clone(), new_last_seen);

            // Filter and process posts as before
            let filtered_posts = posts
                .into_iter()
                .filter(|post| post.subreddit == "wallpaper" || post.subreddit == "wallpapers")
                .collect::<Vec<_>>();

            // Download and create thumbnails
            let paths = client.downloader_post_images(&filtered_posts).await;
            self.create_thumbnails(&paths).await;

            // Update wallpapers
            for post in filtered_posts {
                let wallpaper = Arc::new(Wallpaper {
                    subreddit: post.subreddit,
                    title: post.title,
                    url: post.url,
                    file_name: paths.get(&post.name).unwrap().clone(),
                    name: post.name.clone(),
                });
                self.wallpapers.lock().unwrap().push(wallpaper);
            }
        }

        info!("finished fetching wallpapers from all accounts");
        self.put_clients(clients);
        Ok(())
    }

    async fn create_thumbnails(&self, paths: &HashMap<String, String>) {
        let thumbnails_path = self.wallpaper_path().join("thumbnails");
        if (!thumbnails_path.exists()) {
            create_dir(&thumbnails_path).await.unwrap();
        }
        let mut futures = vec![];
        for file_name in paths.values() {
            let file_name = file_name.to_owned();
            let thumbnails_path = thumbnails_path.clone();
            let file_path = self.wallpaper_path().join(&file_name);
            let future = spawn_blocking(move || {
                let single_path = thumbnails_path.join(&file_name);
                if single_path.exists() {
                    return;
                }
                match Reader::open(&file_path).unwrap().decode() {
                    Ok(image) => {
                        let factor = image.height() as f32 / image.width() as f32;
                        let thumbnail = image.thumbnail(300, (300. * factor) as u32);
                        thumbnail.save(&single_path).unwrap();
                        info!("generated thumbnail {:?}", &single_path);
                    }
                    Err(e) => {
                        warn!("unable to create thumbnail for {file_name} because: {e}")
                    }
                }
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

    pub async fn login(&self, config: Config) -> Result<(), WallpaperError> {
        let client = RedditClient::new(&config).await?;
        {
            let mut clients = self.reddit_clients.lock().unwrap();
            clients.push(client);
        }
        create_dir_all(&config.path)?;
        if config.path.to_str().unwrap() == "" {
            return Err(WallpaperError::NoRootPaths);
        }
        let mut configs = self.config.lock().unwrap();
        if configs.iter().any(|elem| elem.username == config.username) {
            return Err(WallpaperError::UserAlreadyExists);
        }
        configs.push(config);
        Self::save_configs(&configs).map_err(|e| {
            warn!("{e}");
            WallpaperError::Client(ClientError::BadCredetials)
        })?;
        Ok(())
    }

    pub fn is_configured(&self) -> bool {
        self.reddit_clients.lock().unwrap().is_some()
    }
}
