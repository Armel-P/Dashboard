use actix_web::web;
use utoipa::OpenApi;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
    );
}

#[derive(OpenApi)]
#[openapi(
    paths(
    ),
    components(
        schemas(
        )
    )
)]
pub struct ApiDoc;