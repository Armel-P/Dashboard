use actix_web::{HttpRequest, HttpResponse, Responder, http::StatusCode, web, post, put, get,
                middleware::from_fn};

use crate::{
    db::DbPool,
    structs::{
        requests::{MessageResponse, DeleteResponse, UpdateMapRequest, UpdateMapResponse},
        records::MapRecord,
    },
    utils::{http_err, remove_refresh_token_cookie},
    queries::{
        users::{delete_user, map_update, map_get}
    },
    middleware::{jwt_auth},
    api::helpers::get_refresh_token
};

#[utoipa::path(
    post,
    path = "/api/user/delete",
    responses(
        (status = 200, description = "Successfully deleted", content_type = "application/json", body = DeleteResponse),
        (status = 401, description = "Invalid bearer token", content_type = "application/json", body = MessageResponse),
        (status = 500, description = "Internal server error", content_type = "application/json", body = MessageResponse)
    ),
    security(("bearer_auth" = []))
)]
#[post("/delete")]
pub async fn delete(
    req: HttpRequest,
    db: web::Data<DbPool>
) -> impl Responder {
    let mut conn = match db.get().await {
        Ok(conn) => conn,
        Err(_) => return http_err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to connect to database"),
    };

    let refresh_token = match get_refresh_token(&req, &mut conn).await {
        Ok(token_struct) => token_struct,
        Err(http_err) => return http_err
    };

    if delete_user(&mut conn, refresh_token.user_id).await.is_err() {
        return http_err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to disconnect user");
    } // Just delete user, postgre manage the cascade

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
        (status = 200, description = "Successfully updated", content_type = "application/json", body = UpdateMapResponse),
        (status = 401, description = "Invalid bearer token", content_type = "application/json", body = MessageResponse),
        (status = 500, description = "Internal server error", content_type = "application/json", body = MessageResponse)
    ),
    security(("bearer_auth" = []))
)]
#[put("/update-map")]
pub async fn update_map(
    body: web::Json<UpdateMapRequest>,
    req: HttpRequest,
    db: web::Data<DbPool>
) -> impl Responder {
    let mut conn = match db.get().await {
        Ok(conn) => conn,
        Err(_) => return http_err(StatusCode::INTERNAL_SERVER_ERROR, "Filed to connect to database"),
    };

    let refresh_token = match get_refresh_token(&req, &mut conn).await {
        Ok(token_struct) => token_struct,
        Err(http_err) => return http_err
    };

    if map_update(&mut conn, refresh_token.user_id, body.map.clone()).await.is_err() {
        return http_err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to update map");
    }

    HttpResponse::Ok()
        .json(UpdateMapResponse { message: "Map successfully updated".to_string() })
}

#[utoipa::path(
    get,
    path = "/api/user/get-map",
    responses(
        (status = 200, description = "Successfully retrieve", content_type = "application/json"),
        (status = 401, description = "Invalid bearer token", content_type = "application/json", body = MessageResponse),
        (status = 404, description = "No map", content_type = "application/json", body = MessageResponse),
        (status = 500, description = "Internal server error", content_type = "application/json", body = MessageResponse)
    ),
    security(("bearer_auth" = []))
)]
#[get("/get-map")]
pub async fn get_map(
    req: HttpRequest,
    db: web::Data<DbPool>
) -> impl Responder {
    let mut conn = match db.get().await {
        Ok(conn) => conn,
        Err(_) => return http_err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to connect to database"),
    };

    let refresh_token = match get_refresh_token(&req, &mut conn).await {
        Ok(token_struct) => token_struct,
        Err(http_err) => return http_err
    };

    let map_record: MapRecord = match map_get(&mut conn, refresh_token.user_id).await {
        Ok(Some(map)) => map,
        Ok(None) => return http_err(StatusCode::NOT_FOUND, "Not any map saved yet"),
        Err(_) => return http_err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to retrieve map"),
    };

    HttpResponse::Ok().json(map_record.dashboard_map)
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user")
            .wrap(from_fn(jwt_auth))
            .service(delete)
            .service(update_map)
            .service(get_map),
    );
}
