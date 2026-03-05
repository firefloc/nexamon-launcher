use tauri::AppHandle;

use crate::download::client::http_client;
use crate::install::fabric;

#[tauri::command]
pub async fn install_fabric(app: AppHandle) -> Result<(), String> {
    let client = http_client();
    fabric::install_fabric(&client, &app).await?;
    Ok(())
}
