use serde::{Deserialize, Serialize};
use std::fs;

use super::paths;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub ram_mb: u32,
    pub java_path: Option<String>,
    pub close_on_launch: bool,
    #[serde(default)]
    pub auto_accept_configs: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ram_mb: 4096,
            java_path: None,
            close_on_launch: false,
            auto_accept_configs: false,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        let path = paths::settings_path();
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
        let path = paths::settings_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let data = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, data).map_err(|e| e.to_string())?;
        Ok(())
    }
}
