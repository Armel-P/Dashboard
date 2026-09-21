use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub mail: String,
    pub name: String,
    pub password_hash: String,
    pub refresh_token_hash: String
}
pub type RegisterResponse = MessageResponse;
