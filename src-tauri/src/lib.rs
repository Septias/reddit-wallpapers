use json::JsonValue;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
pub mod client;
pub mod wallpaper_manager;

pub const VALID_EXTENSION: [&str; 8] = ["tif", "tiff", "bmp", "jpg", "jpeg", "png", "gif", "raw"];

#[derive(Deserialize, Debug)]
pub struct UserData {
    pub id: String,
    pub name: String,
}


#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    username: String,
    password: String,
    path: PathBuf,
    client_id: String,
    client_secret: String,
}

#[derive(Debug, Serialize, Deserialize, Eq, Clone)]
pub struct Post {
    pub subreddit: String,
    pub title: String,
    pub url: String,
    pub name: String,
}

impl PartialEq for Post {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl std::hash::Hash for Post {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl From<&JsonValue> for Post {
    fn from(jv: &JsonValue) -> Self {
        Self {
            subreddit: jv["data"]["subreddit"].to_string(),
            title: jv["data"]["title"].to_string(),
            url: jv["data"]["url"].to_string(),
            name: jv["data"]["name"].to_string(),
        }
    }
}

#[derive(Deserialize)]
struct TokenInfo {
    access_token: String,
}

#[derive(Error, Debug)]
pub enum WallpaperError {
    #[error("The image-file does not a have a valid image-ending")]
    InvalidEnding,
}
