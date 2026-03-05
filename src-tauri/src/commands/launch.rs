use std::path::PathBuf;
use tauri::AppHandle;

use crate::config::{accounts::AccountStore, paths, profiles::ProfilesData, settings::Settings};
use crate::download::{client::http_client, java, libraries};
use crate::install::fabric;
use crate::launch::{arguments, classpath, process};
use crate::util::progress::{emit_progress, emit_state};

const MC_VERSION: &str = "1.21.1";

#[tauri::command]
pub async fn launch_game(app: AppHandle) -> Result<(), String> {
    let settings = Settings::load();
    let profiles = ProfilesData::load();
    let profile = profiles.selected_profile().ok_or("No profile selected")?;
    let store = AccountStore::load();
    let account = store.account.ok_or("Not logged in")?;

    let client = http_client();

    // 1. Java
    emit_state(&app, "checking_java");
    let java_path = if let Some(ref p) = settings.java_path {
        PathBuf::from(p)
    } else if let Some(p) = java::find_java() {
        p
    } else {
        emit_state(&app, "downloading_java");
        java::download_java(&client, &app).await?
    };

    // 2. Minecraft
    emit_state(&app, "downloading_minecraft");
    crate::commands::minecraft::install_minecraft(app.clone()).await?;

    // 3. Fabric
    emit_state(&app, "installing_fabric");
    let fabric_meta = fabric::install_fabric(&client, &app).await?;

    // 4. packwiz mod sync
    emit_state(&app, "syncing_mods");
    let instance_dir = paths::instance_dir(&profile.id);
    crate::install::packwiz::sync_mods(
        &client,
        &java_path,
        &profile.pack_url,
        &instance_dir,
        &app,
    )
    .await?;

    // 5. Build classpath + args
    emit_state(&app, "launching");
    emit_progress(&app, "Launching...", "", 1.0);

    let version_json_path = paths::versions_dir()
        .join(MC_VERSION)
        .join(format!("{MC_VERSION}.json"));
    let version_data = std::fs::read_to_string(&version_json_path).map_err(|e| e.to_string())?;
    let version_json: serde_json::Value =
        serde_json::from_str(&version_data).map_err(|e| e.to_string())?;

    let mc_libs = libraries::download_libraries(&client, &version_json, &app).await?;
    let client_jar = paths::versions_dir()
        .join(MC_VERSION)
        .join(format!("{MC_VERSION}.jar"));

    let cp = classpath::build_classpath(&mc_libs, &fabric_meta.libraries, &client_jar);

    let args = arguments::build_arguments(
        &version_json,
        &cp,
        &fabric_meta.main_class,
        &account.username,
        &account.uuid,
        &account.access_token,
        &account.xuid,
        settings.ram_mb,
        &instance_dir.to_string_lossy(),
    );

    // 6. Launch
    process::launch_minecraft(
        &java_path,
        &fabric_meta.main_class,
        &args.jvm_args,
        &args.game_args,
        &instance_dir,
        &app,
    )
    .await?;

    Ok(())
}
