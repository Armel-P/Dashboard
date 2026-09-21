use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use serde_json::Value;
use chrono::{DateTime, Utc};

#[derive(FromRow, ToSchema)]
pub struct User {
    id: Uuid,
    mail: String,
    name: String,
    password_hash: String,
    dashboard_map: Option<Value>,
    created_at: DateTime<Utc>
}