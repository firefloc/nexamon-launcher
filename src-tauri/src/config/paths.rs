use std::path::PathBuf;

pub fn data_dir() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
    });
    base.join("nexamon")
}

pub fn shared_dir() -> PathBuf {
    data_dir().join("shared")
}

pub fn instances_dir() -> PathBuf {
    data_dir().join("instances")
}

pub fn java_dir() -> PathBuf {
    shared_dir().join("java")
}

pub fn versions_dir() -> PathBuf {
    shared_dir().join("versions")
}

pub fn assets_dir() -> PathBuf {
    shared_dir().join("assets")
}

pub fn libraries_dir() -> PathBuf {
    shared_dir().join("libraries")
}

pub fn natives_dir() -> PathBuf {
    shared_dir().join("natives")
}

pub fn packwiz_dir() -> PathBuf {
    data_dir().join("packwiz")
}

pub fn settings_path() -> PathBuf {
    data_dir().join("settings.json")
}

pub fn accounts_path() -> PathBuf {
    data_dir().join("accounts.json")
}

pub fn profiles_path() -> PathBuf {
    data_dir().join("profiles.json")
}

pub fn instance_dir(profile_id: &str) -> PathBuf {
    instances_dir().join(profile_id)
}

pub fn ensure_dirs() -> std::io::Result<()> {
    let dirs = [
        data_dir(),
        shared_dir(),
        instances_dir(),
        java_dir(),
        versions_dir(),
        assets_dir(),
        libraries_dir(),
        natives_dir(),
        packwiz_dir(),
    ];
    for d in &dirs {
        std::fs::create_dir_all(d)?;
    }
    Ok(())
}
