use base64::Engine;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

// Match the C# values exactly
const OAUTH_HOST: &str = "account-public-service-prod03.ol.epicgames.com";
const LAUNCHER_CLIENT_ID: &str = "34a02cf8f4414e29b15921876da36f9a";
const LAUNCHER_CLIENT_SECRET: &str = "daafbccc737745039dffe53d94fc76cf";
const USER_AGENT: &str =
    "UELauncher/11.0.1-14907503+++Portal+Release-Live Windows/10.0.19041.1.256.64bit";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpicSession {
    pub access_token: String,
    pub refresh_token: String,
    pub account_id: String,
}

#[derive(Debug, Deserialize)]
struct OAuthResponse {
    access_token: String,
    refresh_token: String,
    account_id: String,
}

#[derive(Debug, Deserialize)]
struct VerifyResponse {
    access_token: Option<String>,
    account_id: String,
}

#[derive(Debug, Deserialize)]
struct GameTokenResponse {
    code: String,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    #[serde(rename = "errorCode")]
    error_code: Option<String>,
    #[serde(rename = "errorMessage")]
    error_message: Option<String>,
}

pub struct EpicApi {
    client: Client,
}

impl EpicApi {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent(USER_AGENT)
                .gzip(true)
                .build()
                .unwrap(),
        }
    }

    fn get_basic_auth() -> String {
        let credentials = format!("{}:{}", LAUNCHER_CLIENT_ID, LAUNCHER_CLIENT_SECRET);
        base64::engine::general_purpose::STANDARD.encode(credentials)
    }

    pub fn get_auth_url() -> String {
        // Must URL-encode the redirect URL, matching C# HttpUtility.UrlEncode
        let redirect_url = format!(
            "https://www.epicgames.com/id/api/redirect?clientId={}&responseType=code",
            LAUNCHER_CLIENT_ID
        );
        let encoded_redirect = urlencoding::encode(&redirect_url);
        format!(
            "https://www.epicgames.com/id/login?redirectUrl={}",
            encoded_redirect
        )
    }

    pub async fn login_with_auth_code(&self, code: &str) -> Result<EpicSession, String> {
        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("token_type", "eg1"),
        ];
        self.oauth_request(&params).await
    }

    pub async fn refresh_session(&self, refresh_token: &str) -> Result<EpicSession, String> {
        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("token_type", "eg1"),
        ];
        self.oauth_request(&params).await
    }

    pub async fn resume_session(&self, access_token: &str) -> Result<EpicSession, String> {
        let url = format!("https://{}/account/api/oauth/verify", OAUTH_HOST);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| format!("Request failed: {e}"))?;

        let body = response.text().await.unwrap_or_default();

        // Check for error in response
        if let Ok(err) = serde_json::from_str::<ErrorResponse>(&body) {
            if err.error_code.is_some() || err.error_message.is_some() {
                return Err(format!(
                    "Session verify failed: {}",
                    err.error_code.unwrap_or_else(|| "Unknown".to_string())
                ));
            }
        }

        let verify: VerifyResponse =
            serde_json::from_str(&body).map_err(|e| format!("Failed to parse response: {e}"))?;

        Ok(EpicSession {
            access_token: access_token.to_string(),
            refresh_token: String::new(), // Verify doesn't return refresh token
            account_id: verify.account_id,
        })
    }

    async fn oauth_request(&self, params: &[(&str, &str)]) -> Result<EpicSession, String> {
        let url = format!("https://{}/account/api/oauth/token", OAUTH_HOST);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Basic {}", Self::get_basic_auth()))
            .form(params)
            .send()
            .await
            .map_err(|e| format!("Request failed: {e}"))?;

        let body = response.text().await.unwrap_or_default();

        // Check for error response
        if let Ok(err) = serde_json::from_str::<ErrorResponse>(&body) {
            if let Some(code) = err.error_code {
                return Err(format!(
                    "Login failed with errorCode: {}. Response: {}",
                    code, body
                ));
            }
        }

        let oauth: OAuthResponse =
            serde_json::from_str(&body).map_err(|e| format!("Failed to parse response: {e}"))?;

        Ok(EpicSession {
            access_token: oauth.access_token,
            refresh_token: oauth.refresh_token,
            account_id: oauth.account_id,
        })
    }

    pub async fn get_game_token(&self, session: &EpicSession) -> Result<String, String> {
        let url = format!("https://{}/account/api/oauth/exchange", OAUTH_HOST);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", session.access_token))
            .send()
            .await
            .map_err(|e| format!("Request failed: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Failed to get game token ({}): {}", status, body));
        }

        let token: GameTokenResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse token: {e}"))?;

        Ok(token.code)
    }
}

// Token storage
fn get_token_path() -> Result<PathBuf, String> {
    let data_dir = dirs::data_local_dir().ok_or("Could not find local data directory")?;
    Ok(data_dir.join("Starlight").join("epic_tokens.json"))
}

pub async fn save_session(session: &EpicSession) -> Result<(), String> {
    let path = get_token_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("Failed to create directory: {e}"))?;
    }
    let json =
        serde_json::to_string_pretty(session).map_err(|e| format!("Failed to serialize: {e}"))?;
    fs::write(&path, json)
        .await
        .map_err(|e| format!("Failed to save: {e}"))?;
    Ok(())
}

pub async fn load_session() -> Option<EpicSession> {
    let path = get_token_path().ok()?;
    let content = fs::read_to_string(&path).await.ok()?;
    serde_json::from_str(&content).ok()
}

pub async fn clear_session() -> Result<(), String> {
    let path = get_token_path()?;
    if path.exists() {
        fs::remove_file(&path)
            .await
            .map_err(|e| format!("Failed to delete: {e}"))?;
    }
    Ok(())
}
