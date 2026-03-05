use reqwest::Client;
use std::path::PathBuf;
use tauri::AppHandle;

use crate::config::paths;
use crate::util::progress::emit_progress;

const MC_VERSION: &str = "1.21.1";
const LOADER_VERSION: &str = "0.18.4";

#[derive(Debug)]
pub struct FabricMeta {
    pub main_class: String,
    pub libraries: Vec<PathBuf>,
}

pub async fn install_fabric(client: &Client, app: &AppHandle) -> Result<FabricMeta, String> {
    emit_progress(app, "Installing Fabric...", "Fetching metadata", 0.0);

    let url = format!(
        "https://meta.fabricmc.net/v2/versions/loader/{MC_VERSION}/{LOADER_VERSION}"
    );
    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let meta: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    let main_class = meta["launcherMeta"]["mainClass"]["client"]
        .as_str()
        .or_else(|| meta["launcherMeta"]["mainClass"].as_str())
        .unwrap_or("net.fabricmc.loader.impl.launch.knot.KnotClient")
        .to_string();

    let mut all_libs = Vec::new();

    // Collect common + client libraries
    for section in ["common", "client"] {
        if let Some(libs) = meta["launcherMeta"]["libraries"][section].as_array() {
            for lib in libs {
                let name = lib["name"].as_str().ok_or("No lib name")?;
                let url = lib["url"].as_str().ok_or("No lib URL")?;
                let path = maven_to_path(name);
                let full_url = format!("{url}{path}");
                let dest = paths::libraries_dir().join(&path);

                if !dest.exists() {
                    super::super::download::client::download_file(
                        client, &full_url, &dest, None, "",
                    )
                    .await?;
                }
                all_libs.push(dest);
            }
        }
    }

    // Also download the loader itself
    let loader_name = meta["loader"]["maven"].as_str().ok_or("No loader maven")?;
    let loader_path = maven_to_path(loader_name);
    let loader_url = format!("https://maven.fabricmc.net/{loader_path}");
    let loader_dest = paths::libraries_dir().join(&loader_path);
    if !loader_dest.exists() {
        super::super::download::client::download_file(
            client, &loader_url, &loader_dest, None, "",
        )
        .await?;
    }
    all_libs.push(loader_dest);

    // Intermediary
    let inter_name = meta["intermediary"]["maven"].as_str().ok_or("No intermediary maven")?;
    let inter_path = maven_to_path(inter_name);
    let inter_url = format!("https://maven.fabricmc.net/{inter_path}");
    let inter_dest = paths::libraries_dir().join(&inter_path);
    if !inter_dest.exists() {
        super::super::download::client::download_file(
            client, &inter_url, &inter_dest, None, "",
        )
        .await?;
    }
    all_libs.push(inter_dest);

    emit_progress(app, "Fabric installed", "", 1.0);

    Ok(FabricMeta {
        main_class,
        libraries: all_libs,
    })
}

fn maven_to_path(name: &str) -> String {
    // net.fabricmc:fabric-loader:0.18.4 -> net/fabricmc/fabric-loader/0.18.4/fabric-loader-0.18.4.jar
    let parts: Vec<&str> = name.split(':').collect();
    if parts.len() < 3 {
        return name.replace(':', "/");
    }
    let group = parts[0].replace('.', "/");
    let artifact = parts[1];
    let version = parts[2];
    format!("{group}/{artifact}/{version}/{artifact}-{version}.jar")
}
