use sqlx::{FromRow};
use utoipa::ToSchema;
use uuid::Uuid;
use serde_json::Value;
use chrono::{DateTime, Utc};
use serde::{Serialize};

#[derive(FromRow, Serialize, ToSchema)]
pub struct User {
    pub id: Uuid,
    pub mail: String,
    pub name: String,
    pub password_hash: String,
    pub dashboard_map: Option<Value>,
    pub created_at: DateTime<Utc>
}

#[derive(FromRow, Serialize, ToSchema)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub created_at: DateTime<Utc>
}
