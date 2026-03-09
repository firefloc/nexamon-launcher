use reqwest::Client;
use serde::{Deserialize, Serialize};

// Nintendo Switch pre-approved client ID (no Azure AD approval needed)
const CLIENT_ID: &str = "00000000441cc96b";
const SCOPE: &str = "service::user.auth.xboxlive.com::MBI_SSL";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    pub user_code: String,
    pub device_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    #[serde(default)]
    pub error: Option<String>,
}

pub async fn start_device_code_flow(client: &Client) -> Result<DeviceCodeResponse, String> {
    let resp = client
        .post("https://login.live.com/oauth20_connect.srf")
        .form(&[
            ("client_id", CLIENT_ID),
            ("scope", SCOPE),
            ("response_type", "device_code"),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let text = resp.text().await.map_err(|e| e.to_string())?;
    log::debug!("Device code response: {text}");

    serde_json::from_str::<DeviceCodeResponse>(&text)
        .map_err(|e| format!("Failed to parse device code response: {e}\nBody: {text}"))
}

pub async fn poll_for_token(
    client: &Client,
    device_code: &str,
    interval: u64,
    expires_in: u64,
) -> Result<(String, String), String> {
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(expires_in);

    loop {
        if start.elapsed() > timeout {
            return Err("Device code expired".into());
        }

        tokio::time::sleep(std::time::Duration::from_secs(interval)).await;

        let resp = client
            .post("https://login.live.com/oauth20_token.srf")
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("client_id", CLIENT_ID),
                ("device_code", device_code),
            ])
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let status = resp.status();
        let text = resp.text().await.map_err(|e| e.to_string())?;
        log::info!("MSA poll status={status}, body_len={}", text.len());
        let token: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;

        if let Some(err) = token.get("error").and_then(|e| e.as_str()) {
            match err {
                "authorization_pending" => continue,
                "slow_down" => {
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    continue;
                }
                _ => return Err(format!("Auth error: {err}")),
            }
        }

        let access_token = token["access_token"]
            .as_str()
            .ok_or("No access_token")?
            .to_string();
        let refresh_token = token["refresh_token"]
            .as_str()
            .ok_or("No refresh_token")?
            .to_string();

        log::info!("MSA token obtained, len={}, starts_with={}", access_token.len(), &access_token[..access_token.len().min(20)]);
        return Ok((access_token, refresh_token));
    }
}

pub async fn refresh_token(client: &Client, refresh: &str) -> Result<(String, String), String> {
    let resp = client
        .post("https://login.live.com/oauth20_token.srf")
        .form(&[
            ("grant_type", "refresh_token"),
            ("client_id", CLIENT_ID),
            ("refresh_token", refresh),
            ("scope", SCOPE),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let token: TokenResponse = resp.json().await.map_err(|e| e.to_string())?;

    if token.error.is_some() {
        return Err("Token refresh failed".into());
    }

    Ok((token.access_token, token.refresh_token))
}
