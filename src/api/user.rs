use actix_web::{HttpRequest, HttpResponse, HttpMessage, Responder, web, post, put,
                middleware::from_fn};
use sqlx::{PgPool};
use uuid::Uuid;
use crate::{
    structs::requests::{MessageResponse, DeleteResponse, UpdateMapRequest, UpdateMapResponse},
    utils::{internal_err, remove_refresh_token_cookie},
    queries::{refresh_tokens::{find_refresh_token}, users::{delete_user, map_update}},
    middleware::{jwt_auth}
};

#[utoipa::path(
    post,
    path = "/api/user/delete",
    responses(
        (status = 204, description = "Successfully deleted", content_type = "application/json", body = DeleteResponse),
        (status = 401, description = "Invalid bearer token", content_type = "application/json", body = MessageResponse),
        (status = 500, description = "Internal server error", content_type = "application/json", body = MessageResponse)
    ),
    security(("bearer_auth" = []))
)]
#[post("/delete")]
pub async fn delete(
    req: HttpRequest,
    db: web::Data<PgPool>
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

    if delete_user(&mut tx, refresh_token.user_id).await.is_err() {
        return internal_err("Failed to disconnect user");
    } // Just delete user, prstgre manage the cascade

    if tx.commit().await.is_err() {
        return internal_err("Failed to commit changes");
    }

    let cookie_remover = remove_refresh_token_cookie();

    HttpResponse::Ok()
        .cookie(cookie_remover)
        .json(DeleteResponse { message: "User successfully deleted".to_string() })
}

#[utoipa::path(
    put,
    path = "/api/user/update-map",
    request_body = UpdateMapRequest,
    responses(
        (status = 204, description = "Successfully updated", content_type = "application/json", body = UpdateMapResponse),
        (status = 401, description = "Invalid bearer token", content_type = "application/json", body = MessageResponse),
        (status = 500, description = "Internal server error", content_type = "application/json", body = MessageResponse)
    ),
    security(("bearer_auth" = []))
)]
#[put("/update-map")]
pub async fn update_map(
    body: web::Json<UpdateMapRequest>,
    req: HttpRequest,
    db: web::Data<PgPool>
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

    if map_update(&mut tx, refresh_token.user_id, body.map.clone()).await.is_err() {
        return internal_err("Failed to update map");
    }

    if tx.commit().await.is_err() {
        return internal_err("Failed to commit changes");
    }

    HttpResponse::Ok()
        .json(UpdateMapResponse { message: "Map successfully updated".to_string() })
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user")
            .wrap(from_fn(jwt_auth))
            .service(delete)
            .service(update_map),
    );
}
