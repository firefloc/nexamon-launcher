use crate::config::settings::Settings;

#[tauri::command]
pub fn get_settings() -> Settings {
    Settings::load()
}

#[tauri::command]
pub fn save_settings(settings: Settings) -> Result<(), String> {
    settings.save()
}

#[tauri::command]
pub fn get_system_ram_mb() -> u64 {
    let mut sys = sysinfo::System::new();
    sys.refresh_memory();
    sys.total_memory() / 1024 / 1024
}
