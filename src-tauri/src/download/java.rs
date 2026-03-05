use reqwest::Client;
use std::path::PathBuf;
use tauri::AppHandle;

use crate::config::paths;
use crate::util::platform;
use crate::util::progress::emit_progress;

pub fn find_java() -> Option<PathBuf> {
    // Check bundled Java first
    let bundled = find_bundled_java();
    if bundled.is_some() {
        return bundled;
    }

    // Check JAVA_HOME
    if let Ok(java_home) = std::env::var("JAVA_HOME") {
        let java = PathBuf::from(&java_home).join("bin").join(platform::java_executable());
        if java.exists() && check_java_version(&java) {
            return Some(java);
        }
    }

    // Check PATH
    if let Ok(output) = std::process::Command::new("java").arg("-version").output() {
        let version = String::from_utf8_lossy(&output.stderr);
        if version.contains("21.") || version.contains("\"21") {
            return Some(PathBuf::from("java"));
        }
    }

    None
}

fn find_bundled_java() -> Option<PathBuf> {
    let java_dir = paths::java_dir();
    if !java_dir.exists() {
        return None;
    }
    // Look for any jdk/jre directory inside
    if let Ok(entries) = std::fs::read_dir(&java_dir) {
        for entry in entries.flatten() {
            let java = entry.path().join("bin").join(platform::java_executable());
            if java.exists() {
                return Some(java);
            }
        }
    }
    None
}

fn check_java_version(java_path: &PathBuf) -> bool {
    if let Ok(output) = std::process::Command::new(java_path).arg("-version").output() {
        let version = String::from_utf8_lossy(&output.stderr);
        return version.contains("21.") || version.contains("\"21");
    }
    false
}

pub async fn download_java(client: &Client, app: &AppHandle) -> Result<PathBuf, String> {
    let os = platform::adoptium_os();
    let arch = platform::adoptium_arch();

    emit_progress(app, "Downloading Java 21...", "Fetching metadata", 0.0);

    let url = format!(
        "https://api.adoptium.net/v3/assets/latest/21/hotspot?os={os}&architecture={arch}&image_type=jre"
    );

    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let assets: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    let asset = assets.as_array()
        .and_then(|a| a.first())
        .ok_or("No Java asset found")?;

    let download_url = asset["binary"]["package"]["link"]
        .as_str()
        .ok_or("No download link")?;

    let filename = asset["binary"]["package"]["name"]
        .as_str()
        .ok_or("No filename")?;

    let dest = std::env::temp_dir().join(filename);

    // Download
    super::client::download_file(client, download_url, &dest, Some(app), "Downloading Java 21...").await?;

    // Extract
    emit_progress(app, "Extracting Java 21...", filename, 0.9);
    let java_dir = paths::java_dir();
    std::fs::create_dir_all(&java_dir).map_err(|e| e.to_string())?;

    if filename.ends_with(".tar.gz") || filename.ends_with(".tgz") {
        extract_tar_gz(&dest, &java_dir)?;
    } else if filename.ends_with(".zip") {
        extract_zip(&dest, &java_dir)?;
    }

    // Clean up temp file
    let _ = std::fs::remove_file(&dest);

    emit_progress(app, "Java 21 installed", "", 1.0);

    find_bundled_java().ok_or_else(|| "Java extraction failed".into())
}

fn extract_tar_gz(archive: &std::path::Path, dest: &std::path::Path) -> Result<(), String> {
    let file = std::fs::File::open(archive).map_err(|e| e.to_string())?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut tar = tar::Archive::new(gz);
    tar.unpack(dest).map_err(|e| e.to_string())?;
    Ok(())
}

fn extract_zip(archive: &std::path::Path, dest: &std::path::Path) -> Result<(), String> {
    let file = std::fs::File::open(archive).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    zip.extract(dest).map_err(|e| e.to_string())?;
    Ok(())
}
