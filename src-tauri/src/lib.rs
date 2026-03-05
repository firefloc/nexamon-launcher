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
    let _ = config::paths::ensure_dirs();

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
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
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::profiles::get_profiles,
            commands::profiles::set_selected_profile,
            commands::profiles::add_profile,
            commands::profiles::remove_profile,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
