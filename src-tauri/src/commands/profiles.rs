use crate::config::profiles::ProfilesData;

#[tauri::command]
pub fn get_profiles() -> ProfilesData {
    ProfilesData::load()
}

#[tauri::command]
pub fn set_selected_profile(id: String) -> Result<(), String> {
    let mut data = ProfilesData::load();
    data.selected = id;
    data.save()
}

#[tauri::command]
pub fn add_profile(
    name: String,
    pack_url: String,
    icon: String,
    description: String,
) -> Result<ProfilesData, String> {
    let mut data = ProfilesData::load();
    data.add_profile(name, pack_url, icon, description);
    data.save()?;
    Ok(data)
}

#[tauri::command]
pub fn remove_profile(id: String) -> Result<ProfilesData, String> {
    let mut data = ProfilesData::load();
    data.remove_profile(&id);
    data.save()?;
    Ok(data)
}
