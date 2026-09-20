use actix_web::web;
use utoipa::OpenApi;

mod about;
mod auth;
mod docs;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
        .service(about::about_json)
        .configure(docs::configure)
    );
}

#[derive(OpenApi)]
#[openapi(
    paths(
        about::about_json
    ),
    components(
        schemas(
        )
    )
)]
pub struct ApiDoc;
