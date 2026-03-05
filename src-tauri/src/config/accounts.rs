use serde::{Deserialize, Serialize};
use std::fs;

use super::paths;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub username: String,
    pub uuid: String,
    pub skin_url: Option<String>,
    pub access_token: String,
    pub refresh_token: String,
    pub xuid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountStore {
    pub account: Option<Account>,
}

impl AccountStore {
    pub fn load() -> Self {
        let path = paths::accounts_path();
        if path.exists() {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(s) = serde_json::from_str(&data) {
                    return s;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = paths::accounts_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let data = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, data).map_err(|e| e.to_string())?;
        Ok(())
    }
}
