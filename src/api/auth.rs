use actix_web::{HttpRequest, HttpResponse, HttpMessage, Responder, http::StatusCode,
    middleware::from_fn, post, web};
use uuid::Uuid;
use chrono::{Utc};
use jsonwebtoken::{EncodingKey};
use diesel::result::{DatabaseErrorKind, Error};

use crate::{
    db::DbPool,
    constants::{REFRESH_TOKEN_LIFETIME_DAYS},
    password::{gen_token, hash_token, hash_password, verify_password, gen_jwt},
    structs::{
        db::{User},
        requests::{MessageResponse, UserInfo, RegisterRequest, RegisterResponse,
                   LoginRequest, LoginResponse, JwtResponse, DisconnectResponse}
        },
    middleware::{jwt_auth},
    queries::{
        users::{insert_user, find_user_by_id, find_user_by_mail},
        refresh_tokens::{insert_refresh_token, delete_refresh_token, find_refresh_token}
    },
    utils::{http_err, build_refresh_token_cookie, remove_refresh_token_cookie}
};

#[utoipa::path(
    post,
    path = "/api/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "Successfully registered", content_type = "application/json", body = RegisterResponse),
        (status = 409, description = "Email already in use", content_type = "application/json", body = MessageResponse),
        (status = 500, description = "Internal server error", content_type = "application/json", body = MessageResponse)
    )
)]
#[post("/register")]
pub async fn register(
    body: web::Json<RegisterRequest>,
    db: web::Data<DbPool>,
    secret: web::Data<Vec<u8>>,
) -> impl Responder {
    let mut conn = match db.get().await {
        Ok(conn) => conn,
        Err(_) => return http_err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to connect to database"),
    };

    let password_hash = match hash_password(&body.password) {
        Ok(pwd_h) => pwd_h,
        Err(_) => return http_err(StatusCode::INTERNAL_SERVER_ERROR, "Fail to hash password")
    };

    let user = match insert_user(&mut conn, body.mail.clone(), body.name.clone(), password_hash)
    .await {
        Ok(id) => id,
            Err(Error::DatabaseError(DatabaseErrorKind::UniqueViolation, info))
                if info.constraint_name() == Some("users_mail_key") =>
                    return http_err(StatusCode::CONFLICT, "An account with this email already exists"),
        Err(_) => return http_err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create account"),
    };

    let token = gen_token();

    let refresh_token = match insert_refresh_token(&mut conn, user.id, token.clone()).await {
        Ok(ref_token) => ref_token,
        Err(_) => return http_err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to store refresh token")
    };

    let jwt = match gen_jwt(refresh_token.id.to_string(), &EncodingKey::from_secret(secret.as_ref())) {
        Ok(token) => token,
        Err(_) => return http_err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate JWT")
    };

    HttpResponse::Created()
        .cookie(build_refresh_token_cookie(refresh_token.id, token))
        .json(RegisterResponse {
        access_token: jwt,
        user: UserInfo {
            id: user.id,
            mail: body.mail.clone(),
            name: body.name.clone(),
        },
    })
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Successfully login", content_type = "application/json", body = LoginResponse),
        (status = 404, description = "User not found", content_type = "application/json", body = MessageResponse),
        (status = 401, description = "Wrong password", content_type = "application/json", body = MessageResponse),
        (status = 500, description = "Internal server error", content_type = "application/json", body = MessageResponse)
    )
)]
#[post("/login")]
pub async fn login(
    body: web::Json<LoginRequest>,
    db: web::Data<DbPool>,
    secret: web::Data<Vec<u8>>,
) -> impl Responder {
    let mut conn = match db.get().await {
        Ok(conn) => conn,
        Err(_) => return http_err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to connect to database"),
    };

    let user: User = match find_user_by_mail(&mut conn, body.mail.clone()).await {
        Ok(Some(user)) => user,
        Ok(None) => return http_err(StatusCode::NOT_FOUND, "Unknown user"),
        Err(_) => return http_err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch user")
    };

    return match verify_password(&body.password, &user.password_hash) {
        Ok(()) => {
            let token = gen_token();

            let refresh_token = match insert_refresh_token(&mut conn, user.id, token.clone()).await {
                Ok(ref_token) => ref_token,
                Err(_) => return http_err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to store refresh token")
            };

            let jwt = match gen_jwt(refresh_token.id.to_string(), &EncodingKey::from_secret(secret.as_ref())) {
                Ok(token) => token,
                Err(_) => return http_err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate JWT")
            };

            HttpResponse::Ok()
                .cookie(build_refresh_token_cookie(refresh_token.id, token))
                .json(LoginResponse {
                access_token: jwt,
                user: UserInfo {
                    id: user.id,
                    mail: user.mail,
                    name: user.name,
                },
            })
        },
        Err(_) => return http_err(StatusCode::UNAUTHORIZED, "Wrong password")
    }
}

#[utoipa::path(
    post,
    path = "/api/auth/get_jwt",
    responses(
        (status = 200, description = "Successfully generated new access token", content_type = "application/json", body = JwtResponse),
        (status = 400, description = "Wrong refresh token format", content_type = "application/json", body = MessageResponse),
        (status = 401, description = "Invalid or expired refresh token", content_type = "application/json", body = MessageResponse),
        (status = 404, description = "Missing refresh token", content_type = "application/json", body = MessageResponse),
        (status = 500, description = "Internal server error", content_type = "application/json", body = MessageResponse)
    )
)]
#[post("/get_jwt")]
pub async fn get_jwt(
    req: HttpRequest,
    db: web::Data<DbPool>,
    secret: web::Data<Vec<u8>>,
) -> impl Responder {
    let cookie = match req.cookie("refresh_token") {
        Some(cookie) => cookie,
        None => return http_err(StatusCode::NOT_FOUND, "Missing refresh token")
    };

    let (id_part, token) = match cookie.value().split_once('.') {
        Some((id_part, token)) => (id_part, token),
        None => return http_err(StatusCode::BAD_REQUEST, "Wrong cookie format")
    };

    let token_id = match Uuid::parse_str(id_part) {
        Ok(token_id) => token_id,
        Err(_) => return http_err(StatusCode::BAD_REQUEST, "Wrong cookie format")
    };

    let mut conn = match db.get().await {
        Ok(conn) => conn,
        Err(_) => return http_err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to connect to database"),
    };

    let refresh_token = match find_refresh_token(&mut conn, token_id).await {
        Ok(Some(token_struct)) => token_struct,
        Ok(None) => return http_err(StatusCode::NOT_FOUND, "Unknown refresh token"),
        Err(_) => return http_err(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
    };

    let user = match find_user_by_id(&mut conn, refresh_token.user_id).await {
        Ok(Some(user_struct)) => user_struct,
        Ok(None) => return http_err(StatusCode::NOT_FOUND, "User not found"),
        Err(_) => return http_err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch user")
    };

    if refresh_token.created_at + chrono::Duration::days(REFRESH_TOKEN_LIFETIME_DAYS) < Utc::now() {
        if let Err(_) = delete_refresh_token(&mut conn, token_id).await {
            return http_err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete expired refresh token");
        }
        return http_err(StatusCode::UNAUTHORIZED, "Refresh token expired");
    }
    if hash_token(&token) != refresh_token.token_hash {
        return http_err(StatusCode::UNAUTHORIZED, "Invalid refresh token");
    }

    let jwt = match gen_jwt(refresh_token.id.to_string(), &EncodingKey::from_secret(secret.as_ref())) {
        Ok(token) => token,
        Err(_) => return http_err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate JWT")
    };

    HttpResponse::Ok().json(JwtResponse { 
        access_token: jwt,
        user: UserInfo {
            id: user.id,
            mail: user.mail,
            name: user.name,
        },
    })
}

#[utoipa::path(
    post,
    path = "/api/auth/disconnect",
    responses(
        (status = 200, description = "Successfully disconnected", content_type = "application/json", body = DisconnectResponse),
        (status = 401, description = "Invalid bearer token", content_type = "application/json", body = MessageResponse),
        (status = 500, description = "Internal server error", content_type = "application/json", body = MessageResponse)
    ),
    security(("bearer_auth" = []))
)]
#[post("/disconnect")]
pub async fn disconnect(
    req: HttpRequest,
    db: web::Data<DbPool>
) -> impl Responder {
    let token_id = match req.extensions().get::<Uuid>() {
        Some(id) => *id,
        None => return http_err(StatusCode::INTERNAL_SERVER_ERROR, "Middleware failed")
    };

    let mut conn = match db.get().await {
        Ok(conn) => conn,
        Err(_) => return http_err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to connect to database"),
    };

    if delete_refresh_token(&mut conn, token_id).await.is_err() {
        return http_err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to disconnect user");
    }

    HttpResponse::Ok()
        .cookie(remove_refresh_token_cookie())
        .json(DisconnectResponse { message: "User successfully disconnected".to_string() })
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
        .service(register)
        .service(login)
        .service(get_jwt)
        .service(
            web::scope("")
                .wrap(from_fn(jwt_auth))
                .service(disconnect),
        ),

    );
}
