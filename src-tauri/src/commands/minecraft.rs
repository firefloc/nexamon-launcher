use tauri::AppHandle;

use crate::config::paths;
use crate::download::{assets, client::http_client, libraries};
use crate::util::progress::emit_progress;

const MC_VERSION: &str = "1.21.1";
const VERSION_MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

#[tauri::command]
pub async fn install_minecraft(app: AppHandle) -> Result<(), String> {
    let client = http_client();

    // 1. Get version manifest
    emit_progress(&app, "Downloading Minecraft...", "Fetching manifest", 0.0);
    let manifest: serde_json::Value = client
        .get(VERSION_MANIFEST_URL)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    // 2. Find our version
    let version_entry = manifest["versions"]
        .as_array()
        .ok_or("No versions")?
        .iter()
        .find(|v| v["id"].as_str() == Some(MC_VERSION))
        .ok_or("Version not found")?;

    let version_url = version_entry["url"].as_str().ok_or("No version URL")?;

    // 3. Download + cache version JSON
    let version_dir = paths::versions_dir().join(MC_VERSION);
    std::fs::create_dir_all(&version_dir).map_err(|e| e.to_string())?;
    let version_json_path = version_dir.join(format!("{MC_VERSION}.json"));

    if !version_json_path.exists() {
        crate::download::client::download_file(
            &client,
            version_url,
            &version_json_path,
            Some(&app),
            "Downloading version info...",
        )
        .await?;
    }

    let version_data = std::fs::read_to_string(&version_json_path).map_err(|e| e.to_string())?;
    let version_json: serde_json::Value =
        serde_json::from_str(&version_data).map_err(|e| e.to_string())?;

    // 4. Download client JAR
    let client_jar = version_dir.join(format!("{MC_VERSION}.jar"));
    if !client_jar.exists() {
        let jar_url = version_json["downloads"]["client"]["url"]
            .as_str()
            .ok_or("No client download URL")?;
        crate::download::client::download_file(
            &client,
            jar_url,
            &client_jar,
            Some(&app),
            "Downloading client...",
        )
        .await?;
    }

    // 5. Download assets
    assets::download_assets(&client, &version_json, &app).await?;

    // 6. Download libraries
    libraries::download_libraries(&client, &version_json, &app).await?;

    emit_progress(&app, "Minecraft installed", "", 1.0);
    Ok(())
}
