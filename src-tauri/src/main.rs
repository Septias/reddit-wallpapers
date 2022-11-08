#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use log::warn;
use reddit_wallpapers::{
    client::ClientError,
    wallpaper_manager::{Wallpaper, WallpaperManager},
    Config, Post, WallpaperError,
};
use std::{sync::Arc};
use tauri::generate_context;

#[tauri::command]
async fn get_all_wallpapers(
    state: tauri::State<'_, Arc<WallpaperManager>>,
) -> Result<Vec<Post>, ClientError> {
    let posts = state.fetch_all_wallpapers().await;
    posts
}

#[tauri::command]
async fn get_cached_wallpapers(
    wm: tauri::State<'_, Arc<WallpaperManager>>,
) -> Result<Vec<Wallpaper>, ()> {
    let mut posts = wm
        .get_cached_wallpapers()
        .await
        .iter()
        .map(|post_arc| (**post_arc).clone())
        .collect::<Vec<_>>();
    posts.reverse();
    Ok(posts)
}

#[tauri::command]
async fn fetch_recent(wm: tauri::State<'_, Arc<WallpaperManager>>) -> Result<(), ClientError> {
    wm.fetch_recent_wallpapers().await
}

#[tauri::command]
async fn select_wallpaper(
    wm: tauri::State<'_, Arc<WallpaperManager>>,
    name: String,
) -> Result<(), ()> {
    wm.set_wallpaper(&name).await;
    Ok(())
}

#[tauri::command]
async fn get_wallpapers_path(wm: tauri::State<'_, Arc<WallpaperManager>>) -> Result<String, ()> {
    Ok(wm.wallpaper_path().to_str().unwrap().to_owned())
}

#[tauri::command]
async fn get_config(wm: tauri::State<'_, Arc<WallpaperManager>>) -> Result<Config, ()> {
    Ok(wm.config.lock().unwrap().clone())
}

#[tauri::command]
async fn set_config(
    wm: tauri::State<'_, Arc<WallpaperManager>>,
    new_config: Config,
) -> Result<(), WallpaperError> {
    wm.set_config(new_config).await
}

#[tauri::command]
fn is_configured(wm: tauri::State<'_, Arc<WallpaperManager>>) -> bool {
    wm.is_configured()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let wm = Arc::new(WallpaperManager::new().await);
    let wm_clone = wm.clone();
    let app = tauri::Builder::default()
        .manage(wm)
        .invoke_handler(tauri::generate_handler![
            get_all_wallpapers,
            get_cached_wallpapers,
            select_wallpaper,
            fetch_recent,
            get_wallpapers_path,
            get_config,
            set_config,
            is_configured
        ])
        .build(generate_context!())
        .expect("error while running tauri application");

    app.run(move |_app_handle, e| {
        if let tauri::RunEvent::Exit { .. } = e {
            wm_clone.save_cache().map_err(|e| warn!("{e}")).ok();
        }
    });
    Ok(())
}
