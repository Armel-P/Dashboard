use actix_web::{HttpResponse, Responder, http::StatusCode, get, web};
use utoipa::OpenApi;
use utoipa_swagger_ui::{Config, SwaggerUi};

use crate::{
    api::ApiDoc,
    utils::http_err
};

#[get("/openapi.json")]
async fn openapi() -> impl Responder {
    match serde_json::to_string(&ApiDoc::openapi()) {
        Ok(json) => HttpResponse::Ok()
            .content_type("application/json")
            .body(json),

        Err(_) => return http_err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate OpenAPI spec"),
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/docs").service(openapi).service(
        SwaggerUi::new("/swagger-ui/{_:.*}").config(Config::new(["/api/docs/openapi.json"])),
    ));
}
