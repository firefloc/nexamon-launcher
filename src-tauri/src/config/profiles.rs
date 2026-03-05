use serde::{Deserialize, Serialize};
use std::fs;

use super::paths;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub pack_url: String,
    pub icon: String,
    pub description: String,
    pub last_played: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilesData {
    pub profiles: Vec<Profile>,
    pub selected: String,
}

impl Default for ProfilesData {
    fn default() -> Self {
        let default_profile = Profile {
            id: "nexamon".into(),
            name: "Nexamon".into(),
            pack_url: "https://firefloc.github.io/nexamon/pack.toml".into(),
            icon: "high".into(),
            description: "Pack complet Nexamon".into(),
            last_played: None,
        };
        Self {
            selected: "nexamon".into(),
            profiles: vec![default_profile],
        }
    }
}

impl ProfilesData {
    pub fn load() -> Self {
        let path = paths::profiles_path();
        if path.exists() {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(s) = serde_json::from_str(&data) {
                    return s;
                }
            }
        }
        let default = Self::default();
        let _ = default.save();
        default
    }

    pub fn save(&self) -> Result<(), String> {
        let path = paths::profiles_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let data = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, data).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn selected_profile(&self) -> Option<&Profile> {
        self.profiles.iter().find(|p| p.id == self.selected)
    }

    pub fn add_profile(&mut self, name: String, pack_url: String, icon: String, description: String) {
        let id = name.to_lowercase().replace(' ', "-");
        let profile = Profile {
            id: id.clone(),
            name,
            pack_url,
            icon,
            description,
            last_played: None,
        };
        self.profiles.push(profile);
        if self.profiles.len() == 1 {
            self.selected = id;
        }
    }

    pub fn remove_profile(&mut self, id: &str) {
        self.profiles.retain(|p| p.id != id);
        if self.selected == id {
            self.selected = self.profiles.first().map(|p| p.id.clone()).unwrap_or_default();
        }
    }
}
