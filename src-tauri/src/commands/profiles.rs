use std::collections::HashMap;

use crate::config::{paths, profiles::ProfilesData};

#[tauri::command]
pub fn get_profiles() -> ProfilesData {
    ProfilesData::load()
}

/// Check install status for all profiles. Returns map of profile_id → "installed" | "not_installed".
#[tauri::command]
pub fn get_pack_statuses() -> HashMap<String, String> {
    let data = ProfilesData::load();
    let mut statuses = HashMap::new();
    for p in &data.profiles {
        let dir = paths::instance_dir(&p.id);
        let mods_dir = dir.join("mods");
        let installed = mods_dir.exists()
            && std::fs::read_dir(&mods_dir)
                .map(|entries| entries.filter(|e| {
                    e.as_ref().map(|e| {
                        e.path().extension().is_some_and(|ext| ext == "jar")
                    }).unwrap_or(false)
                }).count() > 0)
                .unwrap_or(false);
        statuses.insert(
            p.id.clone(),
            if installed { "installed" } else { "not_installed" }.into(),
        );
    }
    statuses
}

/// Delete a profile's instance directory (uninstall the pack).
#[tauri::command]
pub fn uninstall_pack(profile_id: String) -> Result<(), String> {
    let dir = paths::instance_dir(&profile_id);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).map_err(|e| format!("Failed to uninstall: {e}"))?;
    }
    Ok(())
}

#[tauri::command]
pub fn set_selected_profile(id: String) -> Result<(), String> {
    let mut data = ProfilesData::load();
    data.selected = id;
    data.save()
}

#[tauri::command]
pub fn add_profile(
    name: String,
    pack_url: String,
    icon: String,
    description: String,
) -> Result<ProfilesData, String> {
    let mut data = ProfilesData::load();
    data.add_profile(name, pack_url, icon, description);
    data.save()?;
    Ok(data)
}

#[tauri::command]
pub fn remove_profile(id: String) -> Result<ProfilesData, String> {
    let mut data = ProfilesData::load();
    data.remove_profile(&id);
    data.save()?;
    Ok(data)
}

#[tauri::command]
pub fn get_instance_dir() -> Result<String, String> {
    let data = ProfilesData::load();
    let profile = data.selected_profile().ok_or("No profile selected")?;
    let dir = paths::instance_dir(&profile.id);
    Ok(dir.to_string_lossy().to_string())
}

#[tauri::command]
pub fn open_instance_dir() -> Result<(), String> {
    let data = ProfilesData::load();
    let profile = data.selected_profile().ok_or("No profile selected")?;
    let dir = paths::instance_dir(&profile.id);
    if !dir.exists() {
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    let cmd = "xdg-open";
    #[cfg(target_os = "macos")]
    let cmd = "open";
    #[cfg(target_os = "windows")]
    let cmd = "explorer";

    std::process::Command::new(cmd)
        .arg(&dir)
        .spawn()
        .map_err(|e| format!("Failed to open folder: {}", e))?;
    Ok(())
}
