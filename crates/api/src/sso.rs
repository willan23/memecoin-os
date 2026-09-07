use serde::Deserialize;

pub struct OidcConfig {
    pub issuer: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scopes: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
}

#[derive(Debug, Deserialize)]
struct Discovery {
    authorization_endpoint: String,
    token_endpoint: String,
    userinfo_endpoint: String,
    #[serde(default)]
    issuer: Option<String>,
}

/// Env-only gate. Endpoints are filled by [`resolve`].
pub fn config_env() -> Option<(String, String, String, String, String)> {
    let issuer = std::env::var("OIDC_ISSUER").ok().filter(|s| !s.trim().is_empty())?;
    let client_id = std::env::var("OIDC_CLIENT_ID").ok().filter(|s| !s.trim().is_empty())?;
    let client_secret = std::env::var("OIDC_CLIENT_SECRET").ok().filter(|s| !s.trim().is_empty())?;
    let redirect_uri = std::env::var("OIDC_REDIRECT_URI")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| {
            let origin = std::env::var("PUBLIC_WEB_ORIGIN")
                .ok()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "https://memecoin-os.web.app".into());
            format!("{}/v1/auth/oidc/callback", origin.trim_end_matches('/'))
        });
    let scopes = std::env::var("OIDC_SCOPES").unwrap_or_else(|_| "openid email profile".into());
    Some((
        issuer.trim_end_matches('/').into(),
        client_id,
        client_secret,
        redirect_uri,
        scopes,
    ))
}

pub fn configured() -> bool {
    config_env().is_some()
}

/// Fetch OpenID discovery so Google (`accounts.google.com`) and other IdPs work.
pub async fn resolve() -> Result<OidcConfig, String> {
    let (issuer, client_id, client_secret, redirect_uri, scopes) =
        config_env().ok_or_else(|| "OIDC not configured".to_string())?;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;
    let url = format!("{issuer}/.well-known/openid-configuration");
    let res = client.get(&url).send().await.map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        return Err(format!("oidc discovery {}", res.status()));
    }
    let disc: Discovery = res.json().await.map_err(|e| e.to_string())?;
    Ok(OidcConfig {
        issuer: disc.issuer.unwrap_or(issuer),
        client_id,
        client_secret,
        redirect_uri,
        scopes,
        authorization_endpoint: disc.authorization_endpoint,
        token_endpoint: disc.token_endpoint,
        userinfo_endpoint: disc.userinfo_endpoint,
    })
}

pub fn authorize_url(cfg: &OidcConfig, state: &str, nonce: &str) -> String {
    format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&nonce={}",
        cfg.authorization_endpoint,
        urlencoding(&cfg.client_id),
        urlencoding(&cfg.redirect_uri),
        urlencoding(&cfg.scopes),
        urlencoding(state),
        urlencoding(nonce)
    )
}

fn urlencoding(s: &str) -> String {
    s.bytes()
        .flat_map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                vec![b as char]
            }
            _ => format!("%{b:02X}").chars().collect(),
        })
        .collect()
}

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: Option<String>,
    pub id_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UserInfo {
    pub sub: Option<String>,
    pub email: Option<String>,
}

pub async fn exchange_code(cfg: &OidcConfig, code: &str) -> Result<TokenResponse, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;
    let res = client
        .post(&cfg.token_endpoint)
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", cfg.redirect_uri.as_str()),
            ("client_id", cfg.client_id.as_str()),
            ("client_secret", cfg.client_secret.as_str()),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        return Err(format!("oidc token {}", res.status()));
    }
    res.json().await.map_err(|e| e.to_string())
}

pub async fn userinfo(cfg: &OidcConfig, access_token: &str) -> Result<UserInfo, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;
    let res = client
        .get(&cfg.userinfo_endpoint)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        return Err(format!("oidc userinfo {}", res.status()));
    }
    res.json().await.map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unset_oidc_is_not_configured() {
        if std::env::var("OIDC_ISSUER").ok().filter(|s| !s.is_empty()).is_none() {
            assert!(!configured());
        }
    }

    #[test]
    fn authorize_url_contains_state() {
        let cfg = OidcConfig {
            issuer: "https://accounts.google.com".into(),
            client_id: "app".into(),
            client_secret: "s".into(),
            redirect_uri: "http://127.0.0.1:8080/cb".into(),
            scopes: "openid email profile".into(),
            authorization_endpoint: "https://accounts.google.com/o/oauth2/v2/auth".into(),
            token_endpoint: "https://oauth2.googleapis.com/token".into(),
            userinfo_endpoint: "https://openidconnect.googleapis.com/userinfo".into(),
        };
        let u = authorize_url(&cfg, "st", "nn");
        assert!(u.contains("state=st"));
        assert!(u.contains("openid"));
        assert!(u.contains("/o/oauth2/v2/auth"));
        assert!(!u.contains("client_secret"));
    }
}
