use actix_web::{HttpRequest, HttpResponse, Responder, delete, get, http::{StatusCode}, middleware::from_fn, post, web};
use serde_json::Value;
use std::time::Instant;

use crate::{
    api::helpers::get_refresh_token,
    constants::{FRONT_URL, STATE_TTL},
    db::DbPool,
    middleware::jwt_auth,
    queries::oauth_connections::{ delete_oauth_connection, insert_oauth_connection, oauth_connection_exists},
    services::{
        connection::{enc, build_ctx},
        oauth,
        registery::Registry, traits::{AuthKind, ServiceError}
    },
    structs::{
        oauth::{NewOauthConnection, PendingAuth, PendingAuths, ClientCreds},
        requests::{ConnectUrlResponse, ConnectedResponse, GetWidgetUrlInfo,
            MessageResponse, OAuthCallbackQuery, ServiceUrlInfo}
    },
    utils::{http_err, redirect}
};

#[utoipa::path(
    post,
    path = "/api/widgets/connect/{service}",
    params(ServiceUrlInfo),
    responses(
        (status = 200, description = "Successfully connected to service", content_type = "application/json", body = ConnectUrlResponse),
        (status = 400, description = "Bad request", content_type = "application/json", body = MessageResponse),
        (status = 404, description = "Unknown service", content_type = "application/json", body = MessageResponse),
        (status = 409, description = "Already connected", content_type = "application/json", body = MessageResponse),
        (status = 500, description = "Internal error", content_type = "application/json", body = MessageResponse),
    ), security(("bearer_auth" = []))
)]
#[post("/connect/{service}")]
pub async fn connect_service(
    registry: web::Data<Registry>,
    pending: web::Data<PendingAuths>,
    req: HttpRequest,
    path: web::Path<ServiceUrlInfo>,
    db: web::Data<DbPool>,
) -> impl Responder {
    let mut conn = match db.get().await {
        Ok(c) => c,
        Err(_) => return http_err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to connect to database")
    };

    let refresh_token = match get_refresh_token(&req, &mut conn).await {
        Ok(token_struct) => token_struct,
        Err(http_err) => return http_err
    };

    let service_info = path.into_inner();
    let service = match registry.get(&service_info.service) {
        Some(svc) => svc,
        None => return http_err(StatusCode::NOT_FOUND, "Unknown service")
    };

    let AuthKind::OAuth2(cfg) = service.auth() else {
        return http_err(StatusCode::BAD_REQUEST, "Service does not use OAuth");
    };

    match oauth_connection_exists(&mut conn, refresh_token.user_id, &service_info.service).await {
        Ok(false) => {}
        Ok(true) => return http_err(StatusCode::CONFLICT, "Already connected"),
        Err(_) => return http_err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to check connection"),
    }

    let creds = match ClientCreds::from_env(&service_info.service) {
        Ok(c) => c,
        Err(err) => return http_err(StatusCode::INTERNAL_SERVER_ERROR, err.to_string().as_str())
    };

    let state = oauth::random_string(24);
    let (verifier, challenge) = oauth::new_pkce();

    pending.retain(|_, v| v.created_at.elapsed() < STATE_TTL);
    pending.insert(state.clone(), PendingAuth {
        user_id: refresh_token.user_id,
        service: service_info.service,
        code_verifier: verifier,
        created_at: Instant::now(),
    });

    HttpResponse::Ok()
        .json(ConnectUrlResponse { url: oauth::authorize_url(&cfg, &creds, &state, &challenge) })
}

#[utoipa::path(
    get,
    path = "/api/widgets/connect/{service}/callback",
    params(ServiceUrlInfo),
    responses((status = 302, description = "Redirects to the frontend"))
)]
#[get("/connect/{service}/callback")]
pub async fn connect_callback(
    registry: web::Data<Registry>,
    pending: web::Data<PendingAuths>,
    db: web::Data<DbPool>,
    http: web::Data<reqwest::Client>,
    path: web::Path<ServiceUrlInfo>,
    query: web::Query<OAuthCallbackQuery>,
) -> HttpResponse {
    let name = path.into_inner().service;
    let fail = |why: &str| redirect(format!("{FRONT_URL}/profile?error={why}"));

    let Some((_, pa)) = pending.remove(&query.state) else { return fail("invalid_state") };
    if pa.service != name || pa.created_at.elapsed() > STATE_TTL { return fail("invalid_state"); }
    if query.error.is_some() {
        return fail("denied");
    }
    let Some(code) = query.code.as_deref() else { return fail("missing_code") };

    match finish_connection(&registry, &db, http.get_ref(), pa, code).await {
        Ok(()) => redirect(format!("{FRONT_URL}/profile?connected={name}")),
        Err(e) => { tracing::error!("oauth callback failed: {e}");
        fail("exchange_failed") }
    }
}

async fn finish_connection(
    registry: &Registry, db: &DbPool, http: &reqwest::Client, pa: PendingAuth, code: &str,
) -> Result<(), ServiceError> {
    let service = registry.get(&pa.service)
        .ok_or_else(|| ServiceError::Internal("unknown service".into()))?;
    let AuthKind::OAuth2(cfg) = service.auth() else {
        return Err(ServiceError::Internal("not an oauth service".into()));
    };
    let creds = ClientCreds::from_env(&pa.service)?;

    let tok = oauth::exchange_code(http, &cfg, &creds, code, &pa.code_verifier).await?;
    let (provider_user_id, metadata) = service.identify(http, &tok.access_token).await?;

    let mut conn = db.get().await.map_err(|e| ServiceError::Internal(e.to_string()))?;
    insert_oauth_connection(&mut conn, NewOauthConnection {
        user_id: pa.user_id,
        provider: pa.service,
        provider_user_id,
        access_token: enc(&tok.access_token)?,
        refresh_token: tok.refresh_token.as_deref().map(enc).transpose()?,
        expires_at: tok.expires_at(),
        metadata,
    })
    .await
    .map_err(|e| ServiceError::Internal(e.to_string()))?;
    Ok(())
}

#[utoipa::path(get, path = "/api/widgets/connected/{service}", params(ServiceUrlInfo),
    responses((status = 200, body = ConnectedResponse), (status = 404, body = MessageResponse)),
    security(("bearer_auth" = [])))]
#[get("/connected/{service}")]
pub async fn is_connected(
    registry: web::Data<Registry>,
    req: HttpRequest,
    path: web::Path<ServiceUrlInfo>,
    db: web::Data<DbPool>,
) -> impl Responder {
    let mut conn = match db.get().await {
        Ok(c) => c,
        Err(_) => return http_err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to connect to database")
    };

    let refresh_token = match get_refresh_token(&req, &mut conn).await {
        Ok(token_struct) => token_struct,
        Err(http_err) => return http_err
    };

    let name = path.into_inner().service;
    let service = match registry.get(&name) {
        Some(s) => s,
        None => return http_err(StatusCode::NOT_FOUND, "Unknown service"),
    };

    let connected = match service.auth() {
        AuthKind::None => true,
        AuthKind::ApiKey { .. } => false,
        AuthKind::OAuth2(_) => match oauth_connection_exists(&mut conn,
            refresh_token.user_id, &name).await {
            Ok(b) => b,
            Err(_) => return http_err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to check connection"),
        },
    };
    HttpResponse::Ok().json(ConnectedResponse { connected })
}

#[utoipa::path(delete, path = "/api/widgets/connect/{service}", params(ServiceUrlInfo),
    responses((status = 204)), security(("bearer_auth" = [])))]
#[delete("/connect/{service}")]
pub async fn disconnect_service(
    req: HttpRequest,
    path: web::Path<ServiceUrlInfo>,
    db: web::Data<DbPool>,
) -> impl Responder {
    let mut conn = match db.get().await {
        Ok(c) => c,
        Err(_) => return http_err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to connect to database")
    };

    let refresh_token = match get_refresh_token(&req, &mut conn).await {
        Ok(token_struct) => token_struct,
        Err(http_err) => return http_err
    };

    match delete_oauth_connection(&mut conn, refresh_token.user_id, &path.into_inner().service).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(_) => http_err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to disconnect"),
    }
}

#[utoipa::path(
    get,
    path = "/api/widgets/catalog/{service}",
    params(ServiceUrlInfo),
    responses(
        (status = 200, description = "Servic catalog", content_type = "application/json"),
        (status = 401, description = "Invalid bearer token", content_type = "application/json", body = MessageResponse),
        (status = 404, description = "Not found", content_type = "application/json", body = MessageResponse),
        (status = 500, description = "Internal server error", content_type = "application/json", body = MessageResponse)
    ),
    security(("bearer_auth" = []))
)]
#[get("catalog/{service}")]
pub async fn get_catalog(
    registry: web::Data<Registry>,
    req: HttpRequest,
    path: web::Path<ServiceUrlInfo>,
    db: web::Data<DbPool>,
    http: web::Data<reqwest::Client>,
) -> impl Responder {
    let mut conn = match db.get().await {
        Ok(conn) => conn,
        Err(_) => return http_err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to connect to database"),
    };

    let refresh_token = match get_refresh_token(&req, &mut conn).await {
        Ok(token_struct) => token_struct,
        Err(http_err) => return http_err
    };

    let service_info = path.into_inner();
    let service = match registry.get(&service_info.service) {
        Some(svc) => svc,
        None => return http_err(StatusCode::NOT_FOUND, "Unknown service")
    };

    let mut ctx = match build_ctx(&mut conn, http.get_ref(), service.as_ref(), refresh_token.user_id).await {
        Ok(ctx) => ctx,
        Err(resp) => return resp,
    };

    let catalog = match service.describe_catalog(&mut ctx).await {
        Ok(catalog) => catalog,
        Err(err) => return http_err(StatusCode::INTERNAL_SERVER_ERROR, &err.to_string())
    };

    HttpResponse::Ok().json(catalog)
}

// TODO: migrate to QUERY method once actix-web v5.0 and utoipa supports OpenAPI 3.2.0
#[utoipa::path(
    post,
    path = "/api/widgets/{service}/{widget_id}",
    params(GetWidgetUrlInfo),
    request_body = Value,
    responses(
        (status = 200, description = "Widget datas", content_type = "application/json"),
        (status = 401, description = "Invalid bearer token", content_type = "application/json", body = MessageResponse),
        (status = 404, description = "Not found", content_type = "application/json", body = MessageResponse),
        (status = 500, description = "Internal server error", content_type = "application/json", body = MessageResponse)
    ),
    security(("bearer_auth" = []))
)]
#[post("/{service}/{widget_id}")]
pub async fn get_widget(
    registry: web::Data<Registry>,
    req: HttpRequest,
    path: web::Path<GetWidgetUrlInfo>,
    db: web::Data<DbPool>,
    http: web::Data<reqwest::Client>,
    body: web::Json<Value>,
) -> impl Responder {
    let mut conn = match db.get().await {
        Ok(conn) => conn,
        Err(_) => return http_err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to connect to database"),
    };

    let refresh_token = match get_refresh_token(&req, &mut conn).await {
        Ok(token_struct) => token_struct,
        Err(http_err) => return http_err
    };

    let widget_info = path.into_inner();

    let service = match registry.get(&widget_info.service) {
        Some(svc) => svc,
        None => return http_err(StatusCode::NOT_FOUND, "Unknown service")
    };

    let ctx = match build_ctx(&mut conn, http.get_ref(),
        service.as_ref(), refresh_token.user_id).await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    match service.fetch_widget(&ctx, &widget_info.widget_id, body.into_inner()).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(error) => http_err(StatusCode::INTERNAL_SERVER_ERROR, &error.to_string())
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/widgets")
            .service(connect_callback)
            // implement free get widget and free get catalog too for free dashboard
            .service(
                web::scope("")
                    .wrap(from_fn(jwt_auth))
                    .service(connect_service)
                    .service(disconnect_service)
                    .service(is_connected)
                    .service(get_catalog)
                    .service(get_widget),
            ),
    );
}
