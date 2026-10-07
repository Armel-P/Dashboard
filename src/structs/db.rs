use diesel::prelude::*;
use utoipa::ToSchema;
use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use crate::schema::{refresh_tokens, users};

#[derive(Debug, Clone, Queryable, Selectable, Insertable, ToSchema)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,
    pub mail: String,
    pub name: String,
    pub password_hash: String,
    pub dashboard_map: Option<Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, ToSchema)]
#[diesel(table_name = refresh_tokens)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub created_at: DateTime<Utc>,
}
