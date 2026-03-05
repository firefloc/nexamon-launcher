use serde::Serialize;
use std::sync::Mutex;
use tauri::State;

use crate::auth::{microsoft, xbox, minecraft as mc_auth};
use crate::config::accounts::{Account, AccountStore};
use crate::download::client::http_client;

pub struct AuthState {
    pub device_code: Mutex<Option<String>>,
    pub interval: Mutex<u64>,
    pub expires_in: Mutex<u64>,
}

impl Default for AuthState {
    fn default() -> Self {
        Self {
            device_code: Mutex::new(None),
            interval: Mutex::new(5),
            expires_in: Mutex::new(900),
        }
    }
}

#[derive(Serialize)]
pub struct DeviceCodeInfo {
    user_code: String,
    verification_uri: String,
    expires_in: u64,
}

#[derive(Serialize)]
pub struct AccountInfo {
    username: String,
    uuid: String,
    skin_url: Option<String>,
}

#[tauri::command]
pub async fn start_login(state: State<'_, AuthState>) -> Result<DeviceCodeInfo, String> {
    let client = http_client();
    let resp = microsoft::start_device_code_flow(&client).await?;

    *state.device_code.lock().unwrap() = Some(resp.device_code);
    *state.interval.lock().unwrap() = resp.interval;
    *state.expires_in.lock().unwrap() = resp.expires_in;

    Ok(DeviceCodeInfo {
        user_code: resp.user_code,
        verification_uri: resp.verification_uri,
        expires_in: resp.expires_in,
    })
}

#[tauri::command]
pub async fn poll_login(state: State<'_, AuthState>) -> Result<AccountInfo, String> {
    let device_code = state.device_code.lock().unwrap().clone().ok_or("No active login")?;
    let interval = *state.interval.lock().unwrap();
    let expires_in = *state.expires_in.lock().unwrap();

    let client = http_client();

    // 1. Get Microsoft tokens
    let (ms_token, refresh_token) =
        microsoft::poll_for_token(&client, &device_code, interval, expires_in).await?;

    // 2. Xbox Live
    let (xbox_token, user_hash) = xbox::authenticate_xbox_live(&client, &ms_token).await?;

    // 3. XSTS
    let (xsts_token, xuid) = xbox::get_xsts_token(&client, &xbox_token).await?;

    // 4. Minecraft token
    let mc_token = mc_auth::login_with_xbox(&client, &xsts_token, &user_hash).await?;

    // 5. Profile
    let profile = mc_auth::get_profile(&client, &mc_token).await?;

    // Save account
    let account = Account {
        username: profile.name.clone(),
        uuid: profile.id.clone(),
        skin_url: profile.skin_url.clone(),
        access_token: mc_token,
        refresh_token,
        xuid,
    };
    let store = AccountStore {
        account: Some(account),
    };
    store.save()?;

    // Clean up device code
    *state.device_code.lock().unwrap() = None;

    Ok(AccountInfo {
        username: profile.name,
        uuid: profile.id,
        skin_url: profile.skin_url,
    })
}

#[tauri::command]
pub fn get_account() -> Option<AccountInfo> {
    let store = AccountStore::load();
    store.account.map(|a| AccountInfo {
        username: a.username,
        uuid: a.uuid,
        skin_url: a.skin_url,
    })
}

#[tauri::command]
pub fn logout() -> Result<(), String> {
    let store = AccountStore { account: None };
    store.save()
}
