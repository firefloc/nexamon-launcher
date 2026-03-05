use reqwest::Client;
use std::path::PathBuf;
use tauri::AppHandle;

use crate::config::paths;
use crate::util::hash;
use crate::util::platform;
use crate::util::progress::emit_progress;

pub async fn download_libraries(
    client: &Client,
    version_json: &serde_json::Value,
    app: &AppHandle,
) -> Result<Vec<PathBuf>, String> {
    let libraries = version_json["libraries"]
        .as_array()
        .ok_or("No libraries")?;

    let libs_dir = paths::libraries_dir();
    let mut classpath_entries = Vec::new();
    let total = libraries.len();

    for (i, lib) in libraries.iter().enumerate() {
        // Check rules
        if !evaluate_rules(lib) {
            continue;
        }

        if let Some(artifact) = lib.get("downloads").and_then(|d| d.get("artifact")) {
            let path = artifact["path"].as_str().ok_or("No artifact path")?;
            let url = artifact["url"].as_str().ok_or("No artifact URL")?;
            let sha1 = artifact["sha1"].as_str().unwrap_or("");

            let dest = libs_dir.join(path);
            classpath_entries.push(dest.clone());

            if dest.exists() && (sha1.is_empty() || hash::verify_sha1(&dest, sha1)) {
                continue;
            }

            super::client::download_file(client, url, &dest, None, "").await?;
        }

        let progress = (i + 1) as f64 / total as f64;
        let name = lib["name"].as_str().unwrap_or("unknown");
        emit_progress(app, "Downloading libraries...", name, progress);
    }

    Ok(classpath_entries)
}

fn evaluate_rules(lib: &serde_json::Value) -> bool {
    let rules = match lib.get("rules").and_then(|r| r.as_array()) {
        Some(r) => r,
        None => return true, // No rules = always allowed
    };

    let current_os = platform::os_name();
    let mut allowed = false;

    for rule in rules {
        let action = rule["action"].as_str().unwrap_or("allow");
        let has_os = rule.get("os").is_some();

        if !has_os {
            allowed = action == "allow";
            continue;
        }

        let rule_os = rule["os"]["name"].as_str().unwrap_or("");
        if rule_os == current_os {
            allowed = action == "allow";
        }
    }

    allowed
}
