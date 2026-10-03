use sqlx::{FromRow};
use utoipa::ToSchema;
use uuid::Uuid;
use serde_json::Value;

#[derive(FromRow, ToSchema)]
pub struct MapRecord {
    pub dashboard_map: Option<Value>,
}

#[derive(FromRow, ToSchema)]
pub struct IdRecord {
    pub id: Option<Uuid>,
}
