use actix_web::{HttpRequest, HttpResponse, HttpMessage, Responder, get, web,
    middleware::from_fn};
use sqlx::PgPool;
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use crate::{
    services::{
        registery::Registry,
        traits::{AuthKind, ServiceCtx}
    },
    structs::requests::MessageResponse,
    queries::refresh_tokens::find_refresh_token,
    utils::internal_err,
    middleware::jwt_auth
};

#[derive(Deserialize)]
pub struct WidgetInfo {
    service: String,
    widget_id: String
}

#[utoipa::path(
    put,
    path = "/widgets/{service}/{widget_id}",
    responses(
        (status = 204, description = "Widget datas", content_type = "application/json"),
        (status = 401, description = "Invalid bearer token", content_type = "application/json", body = MessageResponse),
        (status = 404, description = "Not found", content_type = "application/json", body = MessageResponse),
        (status = 500, description = "Internal server error", content_type = "application/json", body = MessageResponse)
    ),
    security(("bearer_auth" = []))
)]
#[get("/widgets/{service}/{widget_id}")]
pub async fn get_widget(
    registry: web::Data<Registry>,
    req: HttpRequest,
    path: web::Path<WidgetInfo>,
    db: web::Data<PgPool>,
    http: web::Data<reqwest::Client>,
    query: web::Query<Value>,
) -> impl Responder {
    let token_id = match req.extensions().get::<Uuid>() {
        Some(id) => *id,
        None => return internal_err("Middleware failed")
    };

    let mut tx = match db.begin().await {
        Ok(tx) => tx,
        Err(error) => return internal_err(&error.to_string()),
    };

    let refresh_token = match find_refresh_token(&mut tx, token_id).await {
        Ok(Some(token_struct)) => token_struct,
        Ok(None) => return HttpResponse::NotFound()
            .json(MessageResponse { message: "Refresh token not found".to_string() }),
        Err(_) => return internal_err("Failed to fetch refresh token")
    };

    let widget_info = path.into_inner();

    let service = match registry.get(&widget_info.service) {
        Some(svc) => svc,
        None => return HttpResponse::NotFound()
            .json(MessageResponse { message: "Unknown service".to_string() })
    };

    let ctx = match service.auth() {
        AuthKind::None => ServiceCtx {
            user_id: refresh_token.user_id,
            access_token: None,
            http: http.get_ref().clone(),
        },
        _ => return internal_err("Not implemented yet")
    };

    match service.fetch_widget(&ctx, &widget_info.widget_id, query.into_inner()).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(error) => internal_err(&error.to_string())
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .wrap(from_fn(jwt_auth))
            .service(get_widget)
    );
}