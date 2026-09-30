use sqlx::{FromRow};
use utoipa::ToSchema;
use uuid::Uuid;
use serde_json::Value;
use chrono::{DateTime, Utc};
#[derive(FromRow, ToSchema)]
pub struct User {
    pub id: Uuid,
    pub mail: String,
    pub name: String,
    pub password_hash: String,
    pub dashboard_map: Option<Value>,
    pub created_at: DateTime<Utc>
}

#[derive(FromRow, ToSchema)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub created_at: DateTime<Utc>
}

#[derive(FromRow, ToSchema)]
pub struct OauthConnections {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub provider_user_id: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: String,
    pub metadata: Value
}
