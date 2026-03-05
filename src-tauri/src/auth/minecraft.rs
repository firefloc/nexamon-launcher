use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftProfile {
    pub id: String, // UUID without dashes
    pub name: String,
    pub skin_url: Option<String>,
}

pub async fn login_with_xbox(
    client: &Client,
    xsts_token: &str,
    user_hash: &str,
) -> Result<String, String> {
    let body = json!({
        "identityToken": format!("XBL3.0 x={user_hash};{xsts_token}")
    });

    let resp = client
        .post("https://api.minecraftservices.com/authentication/login_with_xbox")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let data: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    data["access_token"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "No MC access token".into())
}

pub async fn get_profile(client: &Client, mc_token: &str) -> Result<MinecraftProfile, String> {
    let resp = client
        .get("https://api.minecraftservices.com/minecraft/profile")
        .header("Authorization", format!("Bearer {mc_token}"))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let data: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    if let Some(err) = data.get("error") {
        return Err(format!("Profile error: {err}"));
    }

    let id = data["id"].as_str().ok_or("No UUID")?.to_string();
    let name = data["name"].as_str().ok_or("No username")?.to_string();

    let skin_url = data["skins"]
        .as_array()
        .and_then(|skins| skins.first())
        .and_then(|s| s["url"].as_str())
        .map(|s| s.to_string());

    Ok(MinecraftProfile { id, name, skin_url })
}
