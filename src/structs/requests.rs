use serde::{Serialize, Deserialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use serde_json::Value;

#[derive(Serialize, ToSchema)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(Serialize, ToSchema)]
pub struct UserInfo {
    pub id: Uuid,
    pub mail: String,
    pub name: String,
}
#[derive(Serialize, ToSchema)]
pub struct TokenResponse {
    pub access_token: String,
    pub user: UserInfo,
}

#[derive(Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub mail: String,
    pub name: String,
    pub password: String,
}
pub type RegisterResponse = TokenResponse;

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    pub mail: String,
    pub password: String,
}
pub type LoginResponse = TokenResponse;


pub type JwtResponse = TokenResponse;

pub type DisconnectResponse = MessageResponse;

pub type DeleteResponse = MessageResponse;

#[derive(Deserialize, ToSchema)]
pub struct UpdateMapRequest {
    pub map: Value
}
pub type UpdateMapResponse = MessageResponse;

#[derive(Deserialize, IntoParams)]
pub struct ServiceUrlInfo {
    pub service: String,
}

#[derive(Deserialize, IntoParams)]
pub struct GetWidgetUrlInfo {
    pub service: String,
    pub widget_id: String
}

#[derive(Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: Option<String>,
    pub state: String,
    pub error: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct ConnectUrlResponse {
    pub url: String,
}

#[derive(Serialize, ToSchema)]
pub struct ConnectedResponse {
    pub connected: bool,
}
