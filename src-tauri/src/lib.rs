mod auth;
mod commands;
mod config;
mod download;
mod install;
mod launch;
mod util;

use commands::auth::AuthState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Fix white screen on Linux with certain GPU/EGL configurations
    #[cfg(target_os = "linux")]
    {
        if std::env::var("WEBKIT_DISABLE_DMABUF_RENDERER").is_err() {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    }

    let _ = config::paths::ensure_dirs();

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AuthState::default())
        .invoke_handler(tauri::generate_handler![
            commands::auth::start_login,
            commands::auth::poll_login,
            commands::auth::get_account,
            commands::auth::logout,
            commands::java::detect_java,
            commands::java::download_java,
            commands::minecraft::install_minecraft,
            commands::fabric::install_fabric,
            commands::packwiz::sync_mods,
            commands::launch::launch_game,
            commands::launch::prepare_and_sync,
            commands::launch::resolve_configs_and_launch,
            commands::launch::launch_after_sync,
            commands::launch::repair_pack,
            commands::launch::resolve_configs,
            commands::launch::cancel_operation,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::settings::get_system_ram_mb,
            commands::profiles::get_profiles,
            commands::profiles::set_selected_profile,
            commands::profiles::add_profile,
            commands::profiles::remove_profile,
            commands::profiles::get_instance_dir,
            commands::profiles::open_instance_dir,
            commands::profiles::get_pack_statuses,
            commands::profiles::uninstall_pack,
            commands::dev::is_dev_mode,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
