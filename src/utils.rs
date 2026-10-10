use actix_web::{HttpResponse, http::{header, StatusCode}, cookie::{SameSite, Cookie, time::Duration}};
use uuid::Uuid;

use crate::{
    constants::COOKIE_PATH,
    structs::{
        requests::MessageResponse
    }
};

pub fn http_err(status: StatusCode, msg: &str) -> HttpResponse {
    HttpResponse::build(status).json(MessageResponse {
        message: msg.to_string(),
    })
}

pub fn redirect(url: String) -> HttpResponse {
    HttpResponse::Found().insert_header((header::LOCATION, url)).finish()
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
    .same_site(SameSite::Strict)
    .secure(true)
    .finish()
}
