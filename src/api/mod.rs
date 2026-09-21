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

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
        .service(about::about_json)
        .configure(auth::configure)
        .configure(docs::configure)
    );
}

#[derive(OpenApi)]
#[openapi(
    paths(
        about::about_json,
        auth::register,
    ),
    components(
        schemas(
            db::User,
            requests::RegisterRequest, requests::RegisterResponse
        )
    )
)]
pub struct ApiDoc;
