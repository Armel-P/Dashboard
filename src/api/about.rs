use actix_web::{HttpResponse, Responder, get,
    dev::{ConnectionInfo}
};
use utoipa;
use serde_json::json;
use chrono::Utc;

#[utoipa::path(
    get,
    path = "/api/about.json",
    responses(
        (status = 200, description = "Json file containing sever properties", content_type = "application/json"),
        (status = 500, description = "Internal server error")
    )
)]
#[get("/about.json")]
async fn about_json(conn: ConnectionInfo) -> impl Responder {
    let about = json!({
        "client": {
            "host": conn.host(),
        },
        "server": {
            "current_time": Utc::now().timestamp(),
            "services": [
                // Later when services enum implemanted, make a func to loop in the enum
            ]
        },
    });
    match serde_json::to_string(&about) {
        Ok(json) => HttpResponse::Ok()
            .content_type("application/json")
            .body(json),

        Err(err) => HttpResponse::InternalServerError()
            .body(format!("Failed to generate about.json file: {err}")),
    }
}
