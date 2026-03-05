use tauri::AppHandle;

use crate::download::{client::http_client, java};

#[tauri::command]
pub fn detect_java() -> Option<String> {
    java::find_java().map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn download_java(app: AppHandle) -> Result<String, String> {
    let client = http_client();
    let path = java::download_java(&client, &app).await?;
    Ok(path.to_string_lossy().to_string())
}
