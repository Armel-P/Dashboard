use actix_web::web;
use utoipa::OpenApi;
use crate::{
    structs::{
        db, requests
    }
};

mod about;
mod auth;
mod docs;
mod health;
mod user;
mod widget;
pub mod helpers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
        .service(about::about_json)
        .configure(auth::configure)
        .configure(docs::configure)
        .service(health::health)
        .configure(user::configure)
        .configure(widget::configure)
    );
}

#[derive(OpenApi)]
#[openapi(
    paths(
        about::about_json,
        auth::register, auth::login, auth::get_jwt, auth::disconnect,
        health::health,
        user::delete, user::update_map
    ),
    components(
        schemas(
            db::User, db::RefreshToken,
            requests::MessageResponse, requests::UserInfo,
            requests::RegisterRequest, requests::RegisterResponse,
            requests::LoginRequest, requests::LoginResponse,
            requests::JwtResponse, requests::DisconnectResponse,
            requests::DeleteResponse,
            requests::UpdateMapRequest, requests::UpdateMapResponse
        )
    )
)]
pub struct ApiDoc;
