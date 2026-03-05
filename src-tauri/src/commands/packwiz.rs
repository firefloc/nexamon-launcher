use std::path::PathBuf;
use tauri::AppHandle;

use crate::config::{paths, profiles::ProfilesData, settings::Settings};
use crate::download::client::http_client;
use crate::install::packwiz;

#[tauri::command]
pub async fn sync_mods(app: AppHandle) -> Result<(), String> {
    let profiles = ProfilesData::load();
    let profile = profiles
        .selected_profile()
        .ok_or("No profile selected")?;
    let settings = Settings::load();

    let java_path = settings
        .java_path
        .as_ref()
        .map(PathBuf::from)
        .or_else(|| crate::download::java::find_java())
        .ok_or("Java not found")?;

    let instance_dir = paths::instance_dir(&profile.id);
    let client = http_client();

    packwiz::sync_mods(&client, &java_path, &profile.pack_url, &instance_dir, &app).await
}
