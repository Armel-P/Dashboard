use actix_web::{HttpRequest, HttpResponse, HttpMessage, Responder, get, web, post,
    middleware::from_fn};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    db::DbPool,
    services::{
        registery::Registry,
        traits::{AuthKind, ServiceCtx}
    },
    structs::{
        requests::{MessageResponse, GetWidgetUrlInfo, GetCatalogtUrlInfo},
    },
    queries::refresh_tokens::find_refresh_token,
    utils::internal_err,
    middleware::jwt_auth
};

#[utoipa::path(
    get,
    path = "/api/widgets/catalog/{service}",
    responses(
        (status = 200, description = "Servic catalog", content_type = "application/json"),
        (status = 401, description = "Invalid bearer token", content_type = "application/json", body = MessageResponse),
        (status = 404, description = "Not found", content_type = "application/json", body = MessageResponse),
        (status = 500, description = "Internal server error", content_type = "application/json", body = MessageResponse)
    ),
    security(("bearer_auth" = []))
)]
#[get("catalog/{service}")]
pub async fn get_catalog(
    registry: web::Data<Registry>,
    req: HttpRequest,
    path: web::Path<GetCatalogtUrlInfo>,
    db: web::Data<DbPool>,
    http: web::Data<reqwest::Client>,
) -> impl Responder {
    let token_id = match req.extensions().get::<Uuid>() {
        Some(id) => *id,
        None => return internal_err("Middleware failed")
    };

    let mut conn = match db.get().await {
        Ok(conn) => conn,
        Err(error) => return internal_err(&error.to_string()),
    };

    let refresh_token = match find_refresh_token(&mut conn, token_id).await {
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

    let _ = match service.auth() {
        AuthKind::None => ServiceCtx {
            user_id: refresh_token.user_id,
            access_token: None,
            http: http.get_ref().clone(),
        },
        _ => return internal_err("Not implemented yet")
    }; // Let it so a user not connected to a service, cannot know what's inside
    // And in front if a service fail, just skip it, so i'll doesn't display

    HttpResponse::Ok().json(service.describe_catalog())
}

// TODO: migrate to QUERY method once actix-web v5.0 and utoipa supports OpenAPI 3.2.0
#[utoipa::path(
    post,
    path = "/api/widgets/{service}/{widget_id}",
    request_body = Value,
    responses(
        (status = 200, description = "Widget datas", content_type = "application/json"),
        (status = 401, description = "Invalid bearer token", content_type = "application/json", body = MessageResponse),
        (status = 404, description = "Not found", content_type = "application/json", body = MessageResponse),
        (status = 500, description = "Internal server error", content_type = "application/json", body = MessageResponse)
    ),
    security(("bearer_auth" = []))
)]
#[post("/{service}/{widget_id}")]
pub async fn get_widget(
    registry: web::Data<Registry>,
    req: HttpRequest,
    path: web::Path<GetWidgetUrlInfo>,
    db: web::Data<DbPool>,
    http: web::Data<reqwest::Client>,
    body: web::Json<Value>,
) -> impl Responder {
    let token_id = match req.extensions().get::<Uuid>() {
        Some(id) => *id,
        None => return internal_err("Middleware failed")
    };

    let mut conn = match db.get().await {
        Ok(conn) => conn,
        Err(error) => return internal_err(&error.to_string()),
    };

    let refresh_token = match find_refresh_token(&mut conn, token_id).await {
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

    match service.fetch_widget(&ctx, &widget_info.widget_id, body.into_inner()).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(error) => internal_err(&error.to_string())
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/widgets")
            .wrap(from_fn(jwt_auth))
            .service(get_catalog)
            .service(get_widget),
    );
}
