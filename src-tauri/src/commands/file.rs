use std::io::Write;

use tauri::AppHandle;
use reqwest;

#[tauri::command]
pub async fn download_file(app: AppHandle, url: String, save_path: String) -> Result<(), String> {
    let url2 = url.replace(" ", "%20");

    let response = match reqwest::get(url2).await {
        Ok(res) => res,
        Err(err) => return Err(format!("Error: {}", err)),
    };

    if !response.status().is_success() {
        return Err(format!("Error: {}", response.status()));
    }

    let bytes = match response.bytes().await {
        Ok(bytes) => bytes,
        Err(err) => return Err(format!("Error: {}", err)),
    };

    let mut file = match std::fs::File::create(save_path) {
        Ok(file) => file,
        Err(err) => return Err(format!("Error: {}", err)),
    };

    match file.write_all(&bytes) {
        Ok(_) => Ok(()),
        Err(err) => return Err(format!("Error: {}", err)),
    }
}