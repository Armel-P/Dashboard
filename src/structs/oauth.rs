use std::time::Instant;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;
use dashmap::DashMap;

pub type PendingAuths = DashMap<String, PendingAuth>;

pub struct PendingAuth {
    pub user_id: Uuid,
    pub service: String,
    pub code_verifier: String,
    pub created_at: Instant,
}

pub struct NewOauthConnection {
    pub user_id: Uuid,
    pub provider: String,
    pub provider_user_id: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub metadata: Value,
}

pub struct ClientCreds {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Deserialize)]
pub struct TokenResp {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<i64>,
}
