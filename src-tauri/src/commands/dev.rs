#[tauri::command]
pub fn is_dev_mode() -> bool {
    std::env::args().any(|a| a == "-dev" || a == "--dev")
}
