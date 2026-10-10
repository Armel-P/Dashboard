use diesel::prelude::*;
use utoipa::ToSchema;
use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use crate::schema::{users, refresh_tokens, oauth_connections};

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

#[derive(Debug, Clone, Queryable, Selectable, Insertable, ToSchema)]
#[diesel(table_name = oauth_connections)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OauthConnections {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub provider_user_id: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub metadata: Value,
}

