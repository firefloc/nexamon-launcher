use reqwest::Client;
use serde_json::json;

pub async fn authenticate_xbox_live(client: &Client, ms_token: &str) -> Result<(String, String), String> {
    let body = json!({
        "Properties": {
            "AuthMethod": "RPS",
            "SiteName": "user.auth.xboxlive.com",
            "RpsTicket": format!("d={ms_token}")
        },
        "RelyingParty": "http://auth.xboxlive.com",
        "TokenType": "JWT"
    });

    let resp = client
        .post("https://user.auth.xboxlive.com/user/authenticate")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let data: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let token = data["Token"].as_str().ok_or("No Xbox token")?.to_string();
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
