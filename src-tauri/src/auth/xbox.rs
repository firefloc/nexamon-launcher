use reqwest::Client;
use serde_json::json;

pub async fn authenticate_xbox_live(client: &Client, ms_token: &str) -> Result<(String, String), String> {
    let body = json!({
        "Properties": {
            "AuthMethod": "RPS",
            "SiteName": "user.auth.xboxlive.com",
            "RpsTicket": format!("t={ms_token}")
        },
        "RelyingParty": "http://auth.xboxlive.com",
        "TokenType": "JWT"
    });

    let resp = client
        .post("https://user.auth.xboxlive.com/user/authenticate")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("x-xbl-contract-version", "1")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = resp.status();
    let text = resp.text().await.map_err(|e| e.to_string())?;
    log::info!("Xbox Live status={status}, body_len={}, body={}", text.len(), &text[..text.len().min(500)]);

    if !status.is_success() {
        return Err(format!("Xbox Live HTTP {status}: {text}"));
    }

    let data: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("Xbox parse error: {e}\nBody: {text}"))?;
    let token = data["Token"].as_str().ok_or_else(|| format!("No Xbox token. Response: {text}"))?.to_string();
    let uhs = data["DisplayClaims"]["xui"][0]["uhs"]
        .as_str()
        .ok_or("No user hash")?
        .to_string();

    Ok((token, uhs))
}

pub async fn get_xsts_token(client: &Client, xbox_token: &str) -> Result<(String, String), String> {
    let body = json!({
        "Properties": {
            "SandboxId": "RETAIL",
            "UserTokens": [xbox_token]
        },
        "RelyingParty": "rp://api.minecraftservices.com/",
        "TokenType": "JWT"
    });

    let resp = client
        .post("https://xsts.auth.xboxlive.com/xsts/authorize")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let data: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    if let Some(err) = data.get("XErr") {
        return Err(format!("XSTS error: {err}"));
    }

    let token = data["Token"].as_str().ok_or("No XSTS token")?.to_string();
    let uhs = data["DisplayClaims"]["xui"][0]["uhs"]
        .as_str()
        .ok_or("No XSTS user hash")?
        .to_string();

    Ok((token, uhs))
}
