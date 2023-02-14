# Reddit Wallpapers

Application to set wallpapers from reddit as desktop background. Just bookmark your favourite wallpapers from r/wallpapers and they will show up ready to be selected!

## Usage 
Build the project yourself with vue & rust or just download one the prebuild [releases](https://github.com/Septias/reddit-wallpapers/releases).
You then need to create a `user script` for reddit which which allows this program to fetch your saved posts.
Nice tutorial on how to create a `user script` can be found [here](https://github.com/reddit-archive/reddit/wiki/OAuth2-Quick-Start-Example).
Basically it comes down to create a `user script` [here](https://www.reddit.com/prefs/apps).

If you have any further questions feel free to contact me on discord Septias#1614

## Architecture
### Backend
- Rust
- Tauri
- Reqwest for API-requests

### Frontend
- Vue as frontend
- Antfu's vitesse template for vue
- Vite as Devserver
