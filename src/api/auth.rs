use actix_web::{HttpResponse, Responder, post, web};
use utoipa::ToSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value};
use sqlx::{FromRow, PgPool, query_scalar, query};
use uuid::Uuid;
use chrono::Utc;

// TODO: Move table structs in the corresponding file later
#[derive(FromRow, ToSchema)]
pub struct User {
    id: Uuid,
    mail: String,
    name: String,
    password_hash: String,
    dashboard_map: Option<Value>,
    created_at: u16 //Change later maybe

}

// TODO: Move request and response structs in the corresponding file later
#[derive(Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub mail: String,
    pub name: String,
    pub password_hash: String,
    pub refresh_token_hash: String
}

#[derive(Serialize, ToSchema)]
pub struct MessageResponse {
    pub message: String,
}

pub type RegisterResponse = MessageResponse;

#[utoipa::path(
    post,
    path = "/api/register",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "Successfully registered", content_type = "application/json", body = RegisterResponse),
        (status = 409, description = "Email already in use", content_type = "application/json", body = RegisterResponse),
        (status = 500, description = "Internal server error", content_type = "application/json", body = RegisterResponse)
    )
)]
#[post("/register")]
pub async fn register(
    body: web::Json<RegisterRequest>,
    db: web::Data<PgPool>,
) -> impl Responder {
    let mut tx = match db.begin().await {
        Ok(tx) => tx,
        Err(error) => return HttpResponse::InternalServerError()
            .json(RegisterResponse { message: error.to_string() })
    };

    let existing_user: Option<Uuid> = match query_scalar!(
        r#"SELECT id FROM users WHERE mail = $1 LIMIT 1"#,
        body.mail)
    .fetch_optional(&mut *tx)
    .await {
        Ok(user) => user,
        Err(error) => return HttpResponse::InternalServerError()
            .json(RegisterResponse { message: error.to_string() })
    };
    if existing_user.is_some() {
        return HttpResponse::Conflict()
            .json(RegisterResponse {message: "An account with this email already exists".to_string() });
    }

    let user_id = Uuid::new_v4(); //Find a way to be sure to create a new unique one
    let time = Utc::now();

    if let Err(error) = query!(
        r#"INSERT INTO users (
            id, mail, name, password_hash, created_at
        )
        VALUES ($1, $2, $3, $4, $5)"#,
        user_id, body.mail, body.name, body.password_hash, time)
    .execute(&mut *tx)
    .await {
        return HttpResponse::InternalServerError()
            .json(RegisterResponse { message: error.to_string() });
    }

    if let Err(error) = query!(
        r#"
        INSERT INTO refresh_tokens (
            id, token_hash, created_at
        )
        VALUES ($1, $2, $3)"#,
        user_id, body.refresh_token_hash, time
    )
    .execute(&mut *tx)
    .await {
        return HttpResponse::InternalServerError()
            .json(RegisterResponse { message: error.to_string() });
    }

    if let Err(error) = tx.commit().await {
        return HttpResponse::InternalServerError()
            .json(RegisterResponse { message: error.to_string() });
    }

    HttpResponse::Created()
        .json(RegisterResponse { message: "User successfully created".to_string() })
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
        .service(register)
    );
}