use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Duration, Utc};
use rand::RngCore;
use sha2::{Digest, Sha256};

use crate::{
    structs::oauth::{ClientCreds, TokenResp},
    services::traits::{OAuthConfig, ServiceError},
    constants::API_URL
};

impl ClientCreds {
    pub fn from_env(service: &str) -> Result<Self, ServiceError> {
        let prefix = service.to_uppercase();
        let client_id = match std::env::var(format!("{prefix}_CLIENT_ID")) {
            Ok(client_id) => client_id,
            Err(_) => return Err(ServiceError::Internal(format!("Missing env var: {prefix}_CLIENT_ID")))
        };

        let client_secret = match std::env::var(format!("{prefix}_CLIENT_SECRET")) {
            Ok(client_secret) => client_secret,
            Err(_) => return Err(ServiceError::Internal(format!("Missing env var: {prefix}_CLIENT_SECRET")))
        };

        Ok (Self {
            client_id: client_id,
            client_secret: client_secret,
            redirect_uri: format!("{API_URL}/widgets/connect/{service}/callback")
        })
    }
}

pub fn random_string(bytes: usize) -> String {
    let mut buf = vec![0u8; bytes];
    rand::rng().fill_bytes(&mut buf);
    URL_SAFE_NO_PAD.encode(buf)
}

pub fn new_pkce() -> (String, String) {
    let verifier = random_string(32);
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    (verifier, challenge)
}

pub fn authorize_url(cfg: &OAuthConfig, creds: &ClientCreds, state: &str, challenge: &str) -> String {
    let mut params = vec![
        ("client_id", creds.client_id.as_str()),
        ("redirect_uri", creds.redirect_uri.as_str()),
        ("response_type", "code"),
        ("code_challenge", challenge),
        ("code_challenge_method", "S256"),
        ("state", state),
    ];
    let scope = cfg.scopes.join(" ");
    if !scope.is_empty() {
        params.push(("scope", scope.as_str()));
    }
    reqwest::Url::parse_with_params(cfg.auth_url, &params).unwrap().to_string()
}

impl TokenResp {
    pub fn expires_at(&self) -> DateTime<Utc> {
        Utc::now() + Duration::seconds(self.expires_in.unwrap_or(3600))
    }
}

async fn token_request(
    http: &reqwest::Client,
    cfg: &OAuthConfig,
    form: &[(&str, &str)],
) -> Result<TokenResp, ServiceError> {
    let resp = http
        .post(cfg.token_url)
        .header("accept", "application/json; charset=utf-8")
        .form(form)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(ServiceError::Upstream(format!("token endpoint {status}: {body}")));
    }
    Ok(resp.json::<TokenResp>().await?)
}

pub async fn exchange_code(
    http: &reqwest::Client,
    cfg: &OAuthConfig,
    creds: &ClientCreds,
    code: &str,
    verifier: &str,
) -> Result<TokenResp, ServiceError> {
    token_request(http, cfg, &[
        ("grant_type", "authorization_code"),
        ("client_id", &creds.client_id),
        ("client_secret", &creds.client_secret),
        ("redirect_uri", &creds.redirect_uri),
        ("code_verifier", verifier),
        ("code", code),
    ]).await
}

pub async fn refresh(
    http: &reqwest::Client,
    cfg: &OAuthConfig,
    creds: &ClientCreds,
    refresh_token: &str,
) -> Result<TokenResp, ServiceError> {
    token_request(http, cfg, &[
        ("grant_type", "refresh_token"),
        ("client_id", &creds.client_id),
        ("client_secret", &creds.client_secret),
        ("refresh_token", refresh_token),
    ]).await
}
