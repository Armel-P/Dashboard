use actix_web::{HttpRequest, HttpResponse, HttpMessage, Responder,
    middleware::from_fn, post, web};
use sqlx::{PgPool};
use uuid::Uuid;
use chrono::{Utc};
use jsonwebtoken::{EncodingKey};
use crate::{
    constants::{REFRESH_TOKEN_LIFETIME_DAYS},
    password::{gen_token, hash_password, verify_password, gen_jwt},
    structs::{
        db::{User},
        requests::{MessageResponse, UserInfo, RegisterRequest, RegisterResponse,
                   LoginRequest, LoginResponse, JwtResponse, DisconnectResponse}
        },
    middleware::{jwt_auth},
    queries::{
        users::{insert_user, find_user_by_mail},
        refresh_tokens::{insert_refresh_token, find_refresh_token, delete_refresh_token}
    },
    utils::{internal_err, build_refresh_token_cookie, remove_refresh_token_cookie}
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
    db: web::Data<PgPool>,
    secret: web::Data<Vec<u8>>,
) -> impl Responder {
    let mut tx = match db.begin().await {
        Ok(tx) => tx,
        Err(_) => return internal_err("Failed to retrieve database connection")
    };

    let password_hash = match hash_password(&body.password) {
        Ok(pwd_h) => pwd_h,
        Err(_) => return internal_err("Fail to hash password")
    };

    let user = match insert_user(&mut tx, body.mail.clone(), body.name.clone(), password_hash)
    .await {
        Ok(id) => id,
        Err(sqlx::Error::Database(db_error)) if db_error.constraint() == Some("users_mail_key") =>
            return HttpResponse::Conflict()
                .json(MessageResponse { message: "An account with this email already exists".to_string() }),
        Err(_) =>
            return internal_err("Failed to create account"),
    };

    let token = gen_token();

    let refresh_token = match insert_refresh_token(&mut tx, user.id, token.clone()).await {
        Ok(ref_token) => ref_token,
        Err(_) => return internal_err("Failed to store refresh token")
    };

    if let Err(_) = tx.commit().await {
        return internal_err("Fail to commit changes to db");
    }

    let token_cookie = build_refresh_token_cookie(refresh_token.id, token);

    let jwt = match gen_jwt(refresh_token.id.to_string(), &EncodingKey::from_secret(secret.as_ref())) {
        Ok(token) => token,
        Err(_) => return internal_err("Failed to generate JWT")
    };

    HttpResponse::Created()
        .cookie(token_cookie)
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
    db: web::Data<PgPool>,
    secret: web::Data<Vec<u8>>,
) -> impl Responder {
    let mut tx = match db.begin().await {
        Ok(tx) => tx,
        Err(_) => return internal_err("Failed to retrieve database connection")
    };

    let user: User = match find_user_by_mail(&mut tx, body.mail.clone()).await {
        Ok(Some(user)) => user,
        Ok(None) => return HttpResponse::NotFound().json( MessageResponse { message: "Unknown user".to_string() }),
        Err(_) => return internal_err("Failed to fetch user")
    };

    return match verify_password(&body.password, &user.password_hash) {
        Ok(()) => {
            let token = gen_token();

            let refresh_token = match insert_refresh_token(&mut tx, user.id, token.clone()).await {
                Ok(ref_token) => ref_token,
                Err(_) => return internal_err("Failed to store refresh token")
            };

            if let Err(_) = tx.commit().await {
                return internal_err("Fail to commit changes to db");
            }

            let token_cookie = build_refresh_token_cookie(refresh_token.id, token);

            let jwt = match gen_jwt(refresh_token.id.to_string(), &EncodingKey::from_secret(secret.as_ref())) {
                Ok(token) => token,
                Err(_) => return internal_err("Failed to generate JWT")
            };

            HttpResponse::Ok()
                .cookie(token_cookie)
                .json(LoginResponse {
                access_token: jwt,
                user: UserInfo {
                    id: user.id,
                    mail: user.mail,
                    name: user.name,
                },
            })
        },
        Err(_) => {
            HttpResponse::Unauthorized().json( MessageResponse { message: "Wrong password".to_string() })
        }
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
    db: web::Data<PgPool>,
    secret: web::Data<Vec<u8>>,
) -> impl Responder {
    let cookie = match req.cookie("refresh_token") {
        Some(cookie) => cookie,
        None => return HttpResponse::NotFound()
            .json(MessageResponse { message: "Missing refresh token".to_string() })
    };

    let (id_part, token) = match cookie.value().split_once('.') {
        Some((id_part, token)) => (id_part, token),
        None => return HttpResponse::BadRequest()
            .json(MessageResponse { message: "Wrong cookie format".to_string() })
    };

    let token_id = match Uuid::parse_str(id_part) {
        Ok(token_id) => token_id,
        Err(_) => return HttpResponse::BadRequest()
            .json(MessageResponse { message: "Wrong cookie format".to_string() })
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

    if refresh_token.created_at + chrono::Duration::days(REFRESH_TOKEN_LIFETIME_DAYS) < Utc::now() {
        if let Err(_) = delete_refresh_token(&mut tx, token_id).await {
            return internal_err("Failed to delete expired refresh token");
        }
        if let Err(_) = tx.commit().await {
            return internal_err("Failed to commit changes");
        }
        return HttpResponse::Unauthorized()
            .json(MessageResponse { message: "Refresh token expired".to_string() });
    }

    match verify_password(&token, &refresh_token.token_hash) {
        Ok(()) => {
            let jwt = match gen_jwt(refresh_token.id.to_string(), &EncodingKey::from_secret(secret.as_ref())) {
                Ok(token) => token,
                Err(_) => return internal_err("Failed to generate JWT")
            };

            HttpResponse::Ok().json(JwtResponse { access_token: jwt })
        },
        Err(_) => HttpResponse::Unauthorized()
            .json(MessageResponse { message: "Invalid refresh token".to_string() })
    }
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

    if delete_refresh_token(&mut tx, token_id).await.is_err() {
        return internal_err("Failed to disconnect user");
    }

    if tx.commit().await.is_err() {
        return internal_err("Failed to commit changes");
    }

    let cookie_remover = remove_refresh_token_cookie();

    HttpResponse::Ok()
        .cookie(cookie_remover)
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
