#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use app::{
    wallpaper_manager::{Wallpaper, WallpaperManager},
    Post,
};
use std::{fs::read_to_string, path::PathBuf, sync::Arc};
use tauri::generate_context;

#[tauri::command]
async fn get_all_wallpapers(
    state: tauri::State<'_, Arc<WallpaperManager>>,
) -> Result<Vec<Post>, ()> {
    let posts = state.fetch_all_wallpapers().await;
    Ok(posts)
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
async fn fetch_recent(wm: tauri::State<'_, Arc<WallpaperManager>>) -> Result<(), ()> {
    wm.fetch_recent_wallpapers().await;
    Ok(())
}

#[tauri::command]
async fn select_wallpaper(
    wm: tauri::State<'_, Arc<WallpaperManager>>,
    name: String,
) -> Result<(), ()> {
    wm.set_wallpaper(&name).await;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let config = toml::from_str(&read_to_string("./wallpapers.toml").unwrap()).unwrap();

    let wm = Arc::new(if PathBuf::from("./cache.json").exists() {
        WallpaperManager::from_cache(config)
    } else {
        WallpaperManager::new(config)
    });

    let wm_clone = wm.clone();
    let app = tauri::Builder::default()
        .manage(wm)
        .invoke_handler(tauri::generate_handler![
            get_all_wallpapers,
            get_cached_wallpapers,
            select_wallpaper,
            fetch_recent
        ])
        .build(generate_context!())
        .expect("error while running tauri application");

    app.run(move |_app_handle, e| {
        if let tauri::RunEvent::CloseRequested { .. } = e {
            wm_clone.save();
        }
    });
    Ok(())
}
