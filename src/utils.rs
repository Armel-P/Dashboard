use actix_web::{HttpResponse, cookie::{SameSite, Cookie, time::Duration}};
use uuid::Uuid;

use crate::{
    constants::COOKIE_PATH,
    structs::{
        requests::MessageResponse
    }
};

pub fn internal_err(msg: &str) -> HttpResponse {
    HttpResponse::InternalServerError().json(MessageResponse {
        message: msg.to_string(),
    })
}

pub fn build_refresh_token_cookie(token_id: Uuid, secret: String) -> Cookie<'static> {
    Cookie::build("refresh_token", format!("{token_id}.{secret}"))
        .path(COOKIE_PATH)
        .http_only(true)
        .same_site(SameSite::Strict)
        .secure(true)
        .finish()
}

pub fn remove_refresh_token_cookie() -> Cookie<'static> {
    Cookie::build("refresh_token", "")
    .path(COOKIE_PATH)
    .max_age(Duration::ZERO)
    .http_only(true)
    .finish()
}
