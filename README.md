# Reddit Wallpapers

Application to set wallpapers from reddit as desktop-background.

## Usage
- Build the project
- Create a "wallpapers.toml" file as config-file with the following content:
 
```toml
username = "<reddit username>"
password = "<reddit password>"
path = "/home/.../wallpapers/" # absolute path
client_id = "<your reddit-client-key>" # create this at https://www.reddit.com/prefs/apps
client_secret = "your client-secret"
```

Nice tutorial on how to create a user-script: https://github.com/reddit-archive/reddit/wiki/OAuth2-Quick-Start-Example

## Used Technologies
### Backend
- Rust
- Tauri
- Reqwest for API-requests

### Frontend
- Vue as frontend
- Antfu's vitesse template for vue
- Vite as Devserver
