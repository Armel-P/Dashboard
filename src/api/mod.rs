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

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
        .service(about::about_json)
        .configure(auth::configure)
        .configure(docs::configure)
        .service(health::health)
    );
}

#[derive(OpenApi)]
#[openapi(
    paths(
        about::about_json,
        auth::register, auth::login, auth::get_jwt, auth::disconnect,
        health::health
    ),
    components(
        schemas(
            db::User, db::RefreshToken,
            requests::MessageResponse, requests::UserInfo,
            requests::RegisterRequest, requests::RegisterResponse,
            requests::LoginRequest, requests::LoginResponse,
            requests::JwtResponse, requests::DisconnectResponse
        )
    )
)]
pub struct ApiDoc;
