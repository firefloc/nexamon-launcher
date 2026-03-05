use futures_util::StreamExt;
use reqwest::Client;
use std::path::Path;
use tauri::AppHandle;

use crate::util::progress::emit_progress;

pub fn http_client() -> Client {
    Client::builder()
        .user_agent("NexamonLauncher/1.0")
        .build()
        .expect("Failed to create HTTP client")
}

pub async fn download_file(
    client: &Client,
    url: &str,
    dest: &Path,
    app: Option<&AppHandle>,
    label: &str,
) -> Result<(), String> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let resp = client.get(url).send().await.map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {}: {url}", resp.status()));
    }

    let total = resp.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut stream = resp.bytes_stream();
    let mut file = std::fs::File::create(dest).map_err(|e| e.to_string())?;

    use std::io::Write;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        file.write_all(&chunk).map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;

        if let (Some(app), true) = (app, total > 0) {
            let progress = downloaded as f64 / total as f64;
            let filename = dest.file_name().unwrap_or_default().to_string_lossy();
            emit_progress(app, label, &filename, progress);
        }
    }

    Ok(())
}
