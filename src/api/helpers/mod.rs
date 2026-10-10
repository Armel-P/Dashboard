use actix_web::{HttpMessage, HttpRequest, HttpResponse, body::BoxBody, http::StatusCode};
use diesel_async::AsyncPgConnection;
use uuid::Uuid;

use crate::{
    queries::refresh_tokens::find_refresh_token,
    structs::db::RefreshToken,
    utils::http_err,
};

pub async fn get_refresh_token(
    req: &HttpRequest,
    conn: &mut AsyncPgConnection
) -> Result<RefreshToken, HttpResponse<BoxBody>> {
    let token_id = match req.extensions().get::<Uuid>() {
        Some(id) => *id,
        None => return Err(http_err(StatusCode::INTERNAL_SERVER_ERROR, "Middleware failed"))
    };

    return match find_refresh_token(conn, token_id).await {
        Ok(Some(token_struct)) => Ok(token_struct),
        Ok(None) => Err(http_err(StatusCode::NOT_FOUND, "Refresh token not found")),
        Err(_) => Err(http_err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch refresh token"))
    };
}