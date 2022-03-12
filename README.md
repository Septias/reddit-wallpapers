# Reddit Wallpapers

Application to set wallpapers from reddit as desktop-background.

## Usage
- Build the project
- Create a "wallpapers.toml" file as config-file with the following content:
 
```toml
username = "<reddit username>"
password = "<reddit password>"
path = "./wallpapers/" # relative or absolute path
client_id = "<your reddit-client api>"
client_secret = "your client-secret"
```


## Used Technologies
### Backend
- Rust
- Tauri
- Reqwest for API-requests

### Frontend
- Vue as frontend
- Antfu's vitesse template for vue
- Vite as Devserver
