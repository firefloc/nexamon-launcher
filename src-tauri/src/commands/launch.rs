use std::path::PathBuf;
use tauri::AppHandle;

use crate::config::{accounts::AccountStore, paths, profiles::ProfilesData, settings::Settings};
use crate::download::{client::http_client, java, libraries};
use crate::install::{config_guard, fabric, integrity};
use crate::launch::{arguments, classpath, process};
use crate::util::progress::{emit_progress, emit_state};

const MC_VERSION: &str = "1.21.1";

/// Result of sync step: either ready to launch or needs user input on config conflicts.
#[derive(serde::Serialize)]
#[serde(tag = "status")]
pub enum SyncResult {
    /// No config conflicts, can proceed to launch.
    Ready,
    /// Config conflicts detected. Critical ones are force-applied; optional ones need user choice.
    ConfigConflict {
        critical: Vec<String>,
        critical_message: String,
        optional: Vec<String>,
    },
}

/// Steps 1-4: prepare + sync. Returns whether configs need user decision.
#[tauri::command]
pub async fn prepare_and_sync(app: AppHandle) -> Result<SyncResult, String> {
    let settings = Settings::load();
    let profiles = ProfilesData::load();
    let profile = profiles.selected_profile().ok_or("No profile selected")?;

    // Need account to be logged in before we start
    let store = AccountStore::load();
    store.account.as_ref().ok_or("Not logged in")?;

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
    let _fabric_meta = fabric::install_fabric(&client, &app).await?;

    // 4. Pre-sync config backup
    emit_state(&app, "syncing_mods");
    let instance_dir = paths::instance_dir(&profile.id);

    let user_modified = config_guard::pre_sync(&instance_dir)
        .unwrap_or_default();

    // 5. Run packwiz sync
    crate::install::packwiz::sync_mods(
        &client,
        &java_path,
        &profile.pack_url,
        &instance_dir,
        &app,
    )
    .await?;

    // 6. Post-sync: detect conflicts (critical vs optional)
    let update_result = config_guard::post_sync(&instance_dir, &user_modified)
        .unwrap_or_default();

    if update_result.critical.is_empty() && update_result.optional.is_empty() {
        config_guard::accept_new_configs(&instance_dir);
        Ok(SyncResult::Ready)
    } else {
        Ok(SyncResult::ConfigConflict {
            critical: update_result.critical,
            critical_message: update_result.critical_message,
            optional: update_result.optional,
        })
    }
}

/// Resolve config conflict and proceed to launch.
/// `keep_user_configs`: true = restore user's configs, false = keep pack defaults.
#[tauri::command]
pub async fn resolve_configs_and_launch(
    app: AppHandle,
    keep_user_configs: bool,
) -> Result<(), String> {
    let profiles = ProfilesData::load();
    let profile = profiles.selected_profile().ok_or("No profile selected")?;
    let instance_dir = paths::instance_dir(&profile.id);

    if keep_user_configs {
        config_guard::restore_user_configs(&instance_dir)?;
    } else {
        config_guard::accept_new_configs(&instance_dir);
    }

    do_launch(&app).await
}

/// Resolve config conflicts without launching. Used for install-only flow.
#[tauri::command]
pub async fn resolve_configs(keep_user_configs: bool) -> Result<(), String> {
    let profiles = ProfilesData::load();
    let profile = profiles.selected_profile().ok_or("No profile selected")?;
    let instance_dir = paths::instance_dir(&profile.id);

    if keep_user_configs {
        config_guard::restore_user_configs(&instance_dir)?;
    } else {
        config_guard::accept_new_configs(&instance_dir);
    }
    Ok(())
}

/// Launch the game (steps 5-6: classpath + launch). Called after sync is done.
#[tauri::command]
pub async fn launch_after_sync(app: AppHandle) -> Result<(), String> {
    do_launch(&app).await
}

/// Internal launch logic (classpath + args + start process).
async fn do_launch(app: &AppHandle) -> Result<(), String> {
    let settings = Settings::load();
    let profiles = ProfilesData::load();
    let profile = profiles.selected_profile().ok_or("No profile selected")?;
    let store = AccountStore::load();
    let account = store.account.ok_or("Not logged in")?;

    let client = http_client();
    let instance_dir = paths::instance_dir(&profile.id);

    let java_path = if let Some(ref p) = settings.java_path {
        PathBuf::from(p)
    } else if let Some(p) = java::find_java() {
        p
    } else {
        return Err("Java not found".into());
    };

    // Integrity check: fetch remote manifest, verify mods + critical configs
    let integrity_result =
        integrity::verify_integrity(&client, &profile.pack_url, &instance_dir).await;
    if !integrity_result.is_ok() {
        log::warn!("[launch] Integrity issues detected (non-blocking):\n{}", integrity_result.format_error());
    }

    emit_state(app, "launching");
    emit_progress(app, "Launching...", "", 1.0);

    let fabric_meta = fabric::install_fabric(&client, app).await?;

    let version_json_path = paths::versions_dir()
        .join(MC_VERSION)
        .join(format!("{MC_VERSION}.json"));
    let version_data = std::fs::read_to_string(&version_json_path).map_err(|e| e.to_string())?;
    let version_json: serde_json::Value =
        serde_json::from_str(&version_data).map_err(|e| e.to_string())?;

    let mc_libs = libraries::download_libraries(&client, &version_json, app).await?;
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

    process::launch_minecraft(
        &java_path,
        &fabric_meta.main_class,
        &args.jvm_args,
        &args.game_args,
        &instance_dir,
        app,
    )
    .await?;

    Ok(())
}

/// Repair result returned to the frontend.
#[derive(serde::Serialize)]
pub struct RepairResult {
    pub removed_mods: Vec<String>,
    pub removed_datapacks: Vec<String>,
    pub restored_configs: Vec<String>,
    pub resynced: bool,
}

impl RepairResult {
    pub fn is_clean(&self) -> bool {
        self.removed_mods.is_empty()
            && self.removed_datapacks.is_empty()
            && self.restored_configs.is_empty()
            && !self.resynced
    }
}

/// Verify pack integrity and repair any violations.
/// Removes extra mods/datapacks, re-syncs missing ones, restores critical configs.
#[tauri::command]
pub async fn repair_pack(app: AppHandle) -> Result<RepairResult, String> {
    let settings = Settings::load();
    let profiles = ProfilesData::load();
    let profile = profiles.selected_profile().ok_or("No profile selected")?;
    let client = http_client();
    let instance_dir = paths::instance_dir(&profile.id);

    emit_state(&app, "syncing_mods");
    emit_progress(&app, "Verifying integrity...", "", 0.1);

    // 1. Run integrity check
    let check = integrity::verify_integrity(&client, &profile.pack_url, &instance_dir).await;

    let mut result = RepairResult {
        removed_mods: Vec::new(),
        removed_datapacks: Vec::new(),
        restored_configs: Vec::new(),
        resynced: false,
    };

    // 2. Delete extra mods
    let mods_dir = instance_dir.join("mods");
    for name in &check.extra_mods {
        let path = mods_dir.join(name);
        if path.exists() {
            if let Err(e) = std::fs::remove_file(&path) {
                log::warn!("[repair] Failed to remove extra mod {name}: {e}");
            } else {
                result.removed_mods.push(name.clone());
            }
        }
    }

    // 3. Delete extra datapacks
    let dp_dir = instance_dir.join("config").join("paxi").join("datapacks");
    for name in &check.extra_datapacks {
        let path = dp_dir.join(name);
        if path.exists() {
            if let Err(e) = std::fs::remove_file(&path) {
                log::warn!("[repair] Failed to remove extra datapack {name}: {e}");
            } else {
                result.removed_datapacks.push(name.clone());
            }
        }
    }

    // 4. Re-sync if anything was missing or configs were tampered
    let needs_resync = !check.missing_mods.is_empty()
        || !check.missing_datapacks.is_empty()
        || !check.tampered_critical.is_empty();

    if needs_resync {
        emit_progress(&app, "Re-syncing pack...", "", 0.3);

        let java_path = if let Some(ref p) = settings.java_path {
            PathBuf::from(p)
        } else if let Some(p) = java::find_java() {
            p
        } else {
            return Err("Java not found".into());
        };

        // Delete tampered critical configs so packwiz restores them
        let config_dir = instance_dir.join("config");
        for rel_path in &check.tampered_critical {
            let path = config_dir.join(rel_path);
            if path.exists() {
                let _ = std::fs::remove_file(&path);
                result.restored_configs.push(rel_path.clone());
            }
        }

        crate::install::packwiz::sync_mods(
            &client,
            &java_path,
            &profile.pack_url,
            &instance_dir,
            &app,
        )
        .await?;

        result.resynced = true;
    }

    emit_state(&app, "idle");
    emit_progress(&app, "Repair complete", "", 1.0);

    log::info!(
        "[repair] Done: {} mods removed, {} datapacks removed, {} configs restored, resync={}",
        result.removed_mods.len(),
        result.removed_datapacks.len(),
        result.restored_configs.len(),
        result.resynced,
    );

    Ok(result)
}

/// Legacy command kept for compatibility.
#[tauri::command]
pub async fn launch_game(app: AppHandle) -> Result<(), String> {
    let result = prepare_and_sync(app.clone()).await?;
    match result {
        SyncResult::Ready => do_launch(&app).await,
        SyncResult::ConfigConflict { .. } => {
            // Auto-accept new configs if using legacy flow
            let profiles = ProfilesData::load();
            let profile = profiles.selected_profile().ok_or("No profile selected")?;
            config_guard::accept_new_configs(&paths::instance_dir(&profile.id));
            do_launch(&app).await
        }
    }
}
