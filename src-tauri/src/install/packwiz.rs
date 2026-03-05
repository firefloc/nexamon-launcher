use reqwest::Client;
use std::path::Path;
use std::process::Stdio;
use tauri::AppHandle;
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::config::paths;
use crate::util::progress::{emit_log, emit_progress};

const PACKWIZ_INSTALLER_URL: &str =
    "https://github.com/packwiz/packwiz-installer/releases/download/v0.5.14/packwiz-installer.jar";

pub async fn sync_mods(
    client: &Client,
    java_path: &Path,
    pack_url: &str,
    instance_dir: &Path,
    app: &AppHandle,
) -> Result<(), String> {
    let packwiz_jar = paths::packwiz_dir().join("packwiz-installer.jar");

    // Download packwiz-installer if not present
    if !packwiz_jar.exists() {
        emit_progress(app, "Downloading packwiz-installer...", "", 0.0);
        super::super::download::client::download_file(
            client,
            PACKWIZ_INSTALLER_URL,
            &packwiz_jar,
            Some(app),
            "Downloading packwiz-installer...",
        )
        .await?;
    }

    // Ensure instance directory exists
    std::fs::create_dir_all(instance_dir).map_err(|e| e.to_string())?;

    emit_progress(app, "Syncing mods...", "Running packwiz-installer", 0.5);

    // Run packwiz-installer
    let mut child = tokio::process::Command::new(java_path)
        .arg("-jar")
        .arg(&packwiz_jar)
        .arg("-g") // No GUI
        .arg("-s")
        .arg("client") // Client side only
        .arg(pack_url)
        .current_dir(instance_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to run packwiz-installer: {e}"))?;

    // Stream output
    if let Some(stdout) = child.stdout.take() {
        let app_clone = app.clone();
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        tokio::spawn(async move {
            while let Ok(Some(line)) = lines.next_line().await {
                emit_log(&app_clone, &line);
            }
        });
    }

    if let Some(stderr) = child.stderr.take() {
        let app_clone = app.clone();
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        tokio::spawn(async move {
            while let Ok(Some(line)) = lines.next_line().await {
                emit_log(&app_clone, &line);
            }
        });
    }

    let status = child.wait().await.map_err(|e| e.to_string())?;
    if !status.success() {
        return Err(format!("packwiz-installer exited with code: {}", status));
    }

    emit_progress(app, "Mods synced", "", 1.0);
    Ok(())
}
