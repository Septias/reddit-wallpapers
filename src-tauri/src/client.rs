use futures_util::{future::join_all, StreamExt};
use log::{debug, info};
use reqwest::{Client, ClientBuilder};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::{
    self,
    fs::{create_dir_all, File},
    io::AsyncWriteExt,
};

use crate::{Config, Post, TokenInfo, UserData};

pub struct RedditClient {
    client: Client,
    token: Arc<Mutex<Option<String>>>,
    config: Arc<Config>,
}

impl RedditClient {
    pub fn new(config: Arc<Config>) -> Self {
        let client = ClientBuilder::new()
            .user_agent("Wallpaper downloader")
            .build()
            .unwrap();

        Self {
            client,
            token: Arc::new(Mutex::new(None)),
            config,
        }
    }

    async fn get_token(client: &Client, config: &Config) -> Result<String, serde_json::Error> {
        let mut map = HashMap::new();
        map.insert("grant_type", "password");
        map.insert("username", &config.username);
        map.insert("password", &config.password);

        let form_data = map.iter().collect::<Vec<(_, _)>>();

        let resp = client
            .post("https://www.reddit.com/api/v1/access_token")
            .basic_auth(config.client_id.clone(), Some(config.client_secret.clone()))
            .form(&form_data);

        let resp = resp.send().await.unwrap();
        let t: TokenInfo = serde_json::from_str(&resp.text().await.unwrap()).unwrap();
        Ok(t.access_token)
    }

    async fn create_request_with_auth(&self, url: &str, root: &str) -> reqwest::RequestBuilder {
        if self.token.lock().unwrap().is_none() {
            *self.token.lock().unwrap() =
                Some(Self::get_token(&self.client, &self.config).await.unwrap());
        }

        self.client.get(String::from(root) + url).header(
            "Authorization",
            format!("bearer {}", self.token.lock().unwrap().as_ref().unwrap()),
        )
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

    /// fetch all saved posts until `until` is found in one of the reqests
    /// changes until so that it has the id of the newest saved post after
    /// this method finished executing
    pub async fn fetch_saved_until(&self, until: &mut String) -> Vec<Post> {
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
                    &format!("user/{}/saved", self.config.username),
                    &String::from("https://oauth.reddit.com/"),
                )
                .await
                .query(&form)
                .send()
                .await
                .unwrap();

            let content = &saved.text().await.unwrap();

            //fs::write("test.json", content);
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
        *until = new_until;
        info!("loaded {} posts", all_children.len());
        all_children
    }

    pub async fn fetch_all_saved(&self) -> Vec<Post> {
        self.fetch_saved_until(&mut "".to_owned()).await
    }

    pub async fn downloader_post_images(&self, posts: Vec<Arc<Post>>) {
        /* create_dir(&folder)
        .await
        .expect("couldn't create folder for content"); */

        // download all images from posts
        {
            let tasks = posts
                .into_iter()
                .map(|post| self.download_post_image(self.config.path.clone(), post));

            join_all(tasks).await;
        }
    }

    pub async fn download_post_image(
        &self,
        mut path: PathBuf,
        post: Arc<Post>,
    ) -> tauri::async_runtime::TokioJoinHandle<PathBuf> {
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

            path.set_extension(extension);
            info!("saving image {:?} at {:?}", post.title, path);

            let mut file = File::create(&path).await.unwrap();

            let mut body_stream = resp.bytes_stream();
            while let Some(chunk) = body_stream.next().await {
                let chunk = chunk.unwrap();
                file.write_all(&chunk).await.unwrap();
            }
            path
        })
    }
}
