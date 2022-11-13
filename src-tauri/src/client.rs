use futures_util::{future::join_all, StreamExt};
use log::{debug, info, warn};
use reqwest::{Client, ClientBuilder};
use serde::Serialize;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use thiserror::Error;
use tokio::{
    self,
    fs::{create_dir_all, File},
    io::AsyncWriteExt,
};

use crate::{Config, Post, TokenInfo, UserData, WallpaperError, VALID_EXTENSION};

pub struct RedditClient {
    client: Client,
    token: String,
    base_path: PathBuf,
    username: String,
}

#[derive(Error, Debug, Serialize)]
pub enum ClientError {
    #[error("Bad credentials")]
    BadCredetials,
}

async fn get_and_add_to_map(
    post: Arc<Post>,
    map: Arc<Mutex<HashMap<String, String>>>,
    client: &RedditClient,
) {
    let response = client
        .download_post_image(client.base_path.clone(), post.clone())
        .await;

    if let Ok(path_buf) = response {
        map.lock().unwrap().insert(post.name.clone(), path_buf);
    } else {
        warn!("wallpaper error: {:?}", response);
    }
}

impl RedditClient {
    pub async fn new(config: &Config) -> Result<Self, ClientError> {
        let client = ClientBuilder::new()
            .user_agent("Wallpaper downloader")
            .build()
            .unwrap();

        let token = Self::get_token(&client, config).await?;
        Ok(Self {
            client,
            token,
            base_path: config.path.clone(),
            username: config.username.clone(),
        })
    }

    async fn get_token(client: &Client, config: &Config) -> Result<String, ClientError> {
        let mut map = HashMap::new();
        map.insert("grant_type", "password");
        map.insert("username", &config.username);
        map.insert("password", &config.password);

        let form_data = map.iter().collect::<Vec<(_, _)>>();

        let resp = client
            .post("https://www.reddit.com/api/v1/access_token")
            .basic_auth(&config.client_id, Some(&config.client_secret))
            .form(&form_data);

        let resp = resp.send().await.unwrap();
        let t: TokenInfo = serde_json::from_str(&resp.text().await.unwrap())
            .map_err(|_| ClientError::BadCredetials)?;
        Ok(t.access_token)
    }

    async fn create_request_with_auth(&self, url: &str, root: &str) -> reqwest::RequestBuilder {
        self.client
            .get(String::from(root) + url)
            .header("Authorization", format!("bearer {}", self.token))
    }

    pub async fn fetch_userdata(&self) -> UserData {
        let response = self
            .create_request_with_auth("/me", "https://oauth.reddit.com/api/v1")
            .await
            .send()
            .await
            .unwrap();
        serde_json::from_str(&response.text().await.unwrap()).unwrap()
    }

    /// Fetch all saved posts until `until` is found in one of the requests
    /// changes until so that it has the id of the newest saved post after
    /// this method finished executing
    pub async fn fetch_saved_until(&self, until: &str) -> (Vec<Post>, String) {
        let mut all_children = vec![];

        // after is a field accepted by reddit api
        // https://www.reddit.com/dev/api#listings
        let mut after: Option<String> = None;

        // the function sets the accepted variable `until` to the the newest saved post
        let mut new_until = Default::default();

        loop {
            let form: Vec<(&str, String)> = if after.is_none() {
                vec![]
            } else {
                vec![("after", after.as_ref().unwrap().clone())]
            };

            debug!("Requesting saved posts with after: {:?}", after);
            let saved = self
                .create_request_with_auth(
                    &format!("user/{}/saved", self.username),
                    &String::from("https://oauth.reddit.com/"),
                )
                .await
                .query(&form)
                .send()
                .await
                .unwrap();

            let content = &saved.text().await.unwrap();

            // std::fs::write("test.json", content).unwrap();
            let resp = json::parse(content).unwrap();
            let child_array = &resp["data"]["children"];

            // in first iteration set the temp variable to update `until`
            // to the first post in the list, hence the most recent saved one
            if after.is_none() {
                new_until = child_array[0]["data"]["name"].to_string();
            }
            after = Some(resp["data"]["after"].to_string().trim().to_owned());

            if let Some(next_after) = &after {
                if next_after.starts_with("null") {
                    after = None;
                }
            }
            let mut found_last = false;

            // map children-JsonValues to Post-objects
            // only take as many posts until we found the last seen post
            let children = child_array.members().map(Post::from).take_while(|post| {
                if post.name == *until {
                    debug!("stopping at post {}", post.name);
                    found_last = true;
                    false
                } else {
                    true
                }
            });

            all_children.extend(children);

            // stop requesting more if we found the last seen post
            // or there are no more posts to be fetched
            if found_last || after.is_none() {
                break;
            }
        }
        info!("fetched {} posts", all_children.len());
        (all_children, new_until)
    }

    /// gets all posts the user saved
    pub async fn fetch_all_saved_posts(&self) -> Vec<Post> {
        self.fetch_saved_until("").await.0
    }

    /// returns a a hashmap which maps post-ids to the image-paths
    pub async fn downloader_post_images(&self, posts: &[Arc<Post>]) -> HashMap<String, String> {
        let post_to_path = Arc::new(Mutex::new(HashMap::new()));
        let tasks = posts
            .iter()
            .map(|post| get_and_add_to_map(post.clone(), post_to_path.clone(), self));
        join_all(tasks).await;
        Mutex::into_inner(Arc::try_unwrap(post_to_path).unwrap()).unwrap()
    }

    /// download the image contained in the post
    pub async fn download_post_image(
        &self,
        mut path: PathBuf,
        post: Arc<Post>,
    ) -> Result<String, WallpaperError> {
        if !path.is_dir() {
            create_dir_all(&path).await.unwrap();
        }
        path.push(&post.name);
        tokio::spawn(async move {
            let resp = reqwest::get(post.url.clone()).await.unwrap();

            let extension = resp
                .headers()
                .get("content-type")
                .expect("need content-type")
                .to_str()
                .unwrap()
                .split('/')
                .nth(1)
                .unwrap();

            if !VALID_EXTENSION.contains(&extension) {
                return Err(WallpaperError::InvalidEnding);
            }
            path.set_extension(extension);

            if path.exists() {
                info!("Skipping image {:?} as it's already present", post.title);
                return Ok(path.file_name().unwrap().to_str().unwrap().to_owned());
            }
            info!("Saving image {:?} at {:?}", post.title, path);

            let mut file = File::create(&path).await.unwrap();
            let mut body_stream = resp.bytes_stream();
            while let Some(chunk) = body_stream.next().await {
                let chunk = chunk.unwrap();
                file.write_all(&chunk).await.unwrap();
            }

            // TODO: ugly
            Ok(path.file_name().unwrap().to_str().unwrap().to_owned())
        })
        .await
        .unwrap()
    }
}
