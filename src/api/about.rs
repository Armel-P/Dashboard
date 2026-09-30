use actix_web::{HttpResponse, Responder, dev::ConnectionInfo, get, web};
use utoipa;
use serde_json::json;
use chrono::Utc;

use crate::{
    services::registery::Registry,
};

#[utoipa::path(
    get,
    path = "/api/about.json",
    responses(
        (status = 200, description = "Json file containing sever properties", content_type = "application/json"),
        (status = 500, description = "Internal server error")
    )
)]
#[get("/about.json")]
async fn about_json(
    conn: ConnectionInfo,
    registry: web::Data<Registry>
) -> impl Responder {
    HttpResponse::Ok().json(json!({
        "client": { "host": conn.host() },
        "server": {
            "current_time": Utc::now().timestamp(),
            "services": registry.all().map(|s| s.describe()).collect::<Vec<_>>(),
        },
    }))
}
