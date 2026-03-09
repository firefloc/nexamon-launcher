use std::path::PathBuf;
use tauri::AppHandle;

use crate::auth::{microsoft, xbox, minecraft as mc_auth};
use crate::config::{accounts::{Account, AccountStore}, paths, profiles::ProfilesData, settings::Settings};
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

/// Cancel any in-progress operation (prepare_and_sync, repair, etc.)
#[tauri::command]
pub fn cancel_operation() {
    crate::util::cancel::request_cancel();
}

/// Steps 1-4: prepare + sync. Returns whether configs need user decision.
#[tauri::command]
pub async fn prepare_and_sync(app: AppHandle) -> Result<SyncResult, String> {
    crate::util::cancel::reset();
    crate::install::fabric::clear_cache();

    let settings = Settings::load();
    let profiles = ProfilesData::load();
    let profile = profiles.selected_profile().ok_or("No profile selected")?;

    // Need account to be logged in before we start
    let store = AccountStore::load();
    store.account.as_ref().ok_or("Not logged in")?;

    let client = http_client();

    // Global progress: Java=0-5%, Minecraft=5-15%, Fabric=15-20%, Sync=20-95%, Done=95-100%

    // 1. Java (0-5%)
    emit_state(&app, "checking_java");
    emit_progress(&app, "Checking Java...", "", 0.0);
    crate::util::cancel::check_cancelled()?;
    let java_path = if let Some(ref p) = settings.java_path {
        PathBuf::from(p)
    } else if let Some(p) = java::find_java() {
        p
    } else {
        emit_state(&app, "downloading_java");
        emit_progress(&app, "Downloading Java...", "", 0.02);
        java::download_java(&client, &app).await?
    };
    emit_progress(&app, "Java ready", "", 0.05);

    // 2. Minecraft (5-15%)
    crate::util::cancel::check_cancelled()?;
    emit_state(&app, "downloading_minecraft");
    emit_progress(&app, "Downloading Minecraft...", "", 0.05);
    crate::commands::minecraft::install_minecraft(app.clone()).await?;
    emit_progress(&app, "Minecraft ready", "", 0.15);

    // 3. Fabric (15-20%)
    crate::util::cancel::check_cancelled()?;
    emit_state(&app, "installing_fabric");
    emit_progress(&app, "Installing Fabric...", "", 0.15);
    let _fabric_meta = fabric::install_fabric(&client, &app).await?;
    emit_progress(&app, "Fabric ready", "", 0.20);

    // 4. Pre-sync config backup + packwiz sync (20-95%)
    crate::util::cancel::check_cancelled()?;
    emit_state(&app, "syncing_mods");
    emit_progress(&app, "Syncing mods...", "", 0.20);
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

    // Refresh auth tokens before launch to avoid expired session
    emit_progress(app, "Refreshing session...", "", 0.96);
    let account = match refresh_auth_tokens(&client, &account).await {
        Ok(refreshed) => {
            log::info!("[launch] Auth tokens refreshed for {}", refreshed.username);
            // Save refreshed tokens
            let new_store = AccountStore { account: Some(refreshed.clone()) };
            if let Err(e) = new_store.save() {
                log::warn!("[launch] Failed to save refreshed tokens: {e}");
            }
            refreshed
        }
        Err(e) => {
            log::warn!("[launch] Token refresh failed, using existing tokens: {e}");
            account
        }
    };

    let instance_dir = paths::instance_dir(&profile.id);

    let java_path = if let Some(ref p) = settings.java_path {
        PathBuf::from(p)
    } else if let Some(p) = java::find_java() {
        p
    } else {
        return Err("Java not found".into());
    };

    // Integrity check: fetch remote manifest, verify mods + critical configs
    emit_progress(app, "Verifying integrity...", "", 0.97);
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

/// Refresh the full auth chain: MS → Xbox → XSTS → MC token.
async fn refresh_auth_tokens(client: &reqwest::Client, account: &Account) -> Result<Account, String> {
    // 1. Refresh Microsoft token
    let (ms_token, new_refresh) = microsoft::refresh_token(client, &account.refresh_token).await?;

    // 2. Xbox Live
    let (xbox_token, _uhs) = xbox::authenticate_xbox_live(client, &ms_token).await?;

    // 3. XSTS
    let (xsts_token, user_hash) = xbox::get_xsts_token(client, &xbox_token).await?;

    // 4. Minecraft token
    let mc_token = mc_auth::login_with_xbox(client, &xsts_token, &user_hash).await?;

    // 5. Fetch profile to get current username
    let profile = mc_auth::get_profile(client, &mc_token).await?;

    Ok(Account {
        username: profile.name,
        uuid: profile.id,
        skin_url: profile.skin_url,
        access_token: mc_token,
        refresh_token: new_refresh,
        xuid: account.xuid.clone(),
    })
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
