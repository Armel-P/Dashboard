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
        auth::register, auth::verify, auth::login, auth::get_jwt, auth::disconnect,
        health::health,
        user::delete, user::update_map, user::get_map,
        widget::connect_service, widget::connect_callback, widget::is_connected, widget::disconnect_service,
        widget::get_catalog, widget::get_widget, widget::get_catalog_free, widget::get_widget_free
    ),
    components(
        schemas(
            db::User, db::RefreshToken,
            requests::MessageResponse, requests::UserInfo,
            requests::RegisterRequest, requests::VerifyRequest,
            requests::LoginRequest, requests::LoginResponse,
            requests::JwtResponse, requests::DisconnectResponse,
            requests::DeleteResponse,
            requests::UpdateMapRequest, requests::UpdateMapResponse,
        )
    )
)]
pub struct ApiDoc;
