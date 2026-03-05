use reqwest::Client;
use std::path::Path;
use tauri::AppHandle;

use crate::config::paths;
use crate::util::hash;
use crate::util::progress::emit_progress;

pub async fn download_assets(
    client: &Client,
    version_json: &serde_json::Value,
    app: &AppHandle,
) -> Result<(), String> {
    let asset_index_url = version_json["assetIndex"]["url"]
        .as_str()
        .ok_or("No asset index URL")?;
    let asset_index_id = version_json["assetIndex"]["id"]
        .as_str()
        .ok_or("No asset index ID")?;

    // Download asset index
    let index_dir = paths::assets_dir().join("indexes");
    std::fs::create_dir_all(&index_dir).map_err(|e| e.to_string())?;
    let index_path = index_dir.join(format!("{asset_index_id}.json"));

    if !index_path.exists() {
        super::client::download_file(client, asset_index_url, &index_path, Some(app), "Downloading asset index...").await?;
    }

    // Parse asset index
    let index_data = std::fs::read_to_string(&index_path).map_err(|e| e.to_string())?;
    let index: serde_json::Value = serde_json::from_str(&index_data).map_err(|e| e.to_string())?;
    let objects = index["objects"].as_object().ok_or("No objects in asset index")?;

    let objects_dir = paths::assets_dir().join("objects");
    let total = objects.len();
    let mut downloaded = 0;

    for (_name, obj) in objects {
        let obj_hash = obj["hash"].as_str().ok_or("No hash in asset")?;
        let prefix = &obj_hash[..2];
        let obj_path = objects_dir.join(prefix).join(obj_hash);

        if obj_path.exists() && hash::verify_sha1(&obj_path, obj_hash) {
            downloaded += 1;
            continue;
        }

        let url = format!("https://resources.download.minecraft.net/{prefix}/{obj_hash}");
        std::fs::create_dir_all(obj_path.parent().unwrap()).map_err(|e| e.to_string())?;
        super::client::download_file(client, &url, &obj_path, None, "").await?;
        downloaded += 1;

        let progress = downloaded as f64 / total as f64;
        emit_progress(app, "Downloading assets...", &format!("{downloaded}/{total}"), progress);
    }

    Ok(())
}
