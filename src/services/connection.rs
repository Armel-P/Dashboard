use actix_web::{HttpResponse, http::StatusCode};
use chrono::{Duration, Utc};
use diesel_async::AsyncPgConnection;
use uuid::Uuid;

use crate::{
    utils::http_err,
    crypto::{encrypt, decrypt},
    queries::oauth_connections::{find_oauth_connection, tokens_update},
    services::{
        oauth,
        traits::{AuthKind, OAuthConfig, Service, ServiceCtx, ServiceError},
    },
    structs::{oauth::ClientCreds},
};

pub fn enc(s: &str) -> Result<String, ServiceError> {
    encrypt(s).map_err(|e| ServiceError::Internal(e.to_string()))
}
pub fn dec(s: &str) -> Result<String, ServiceError> {
    decrypt(s).map_err(|e| ServiceError::Internal(e.to_string()))
}
fn db_err(e: diesel::result::Error) -> ServiceError {
    ServiceError::Internal(e.to_string())
}

pub async fn build_ctx(
    conn: &mut AsyncPgConnection,
    http: &reqwest::Client,
    service: &dyn Service,
    user_id: Uuid,
) -> Result<ServiceCtx, HttpResponse> {
    let access_token = match service.auth() {
        AuthKind::None => None,
        AuthKind::ApiKey { .. } => {
            return Err(http_err(StatusCode::INTERNAL_SERVER_ERROR, "Api key auth not implemented"))
        }
        AuthKind::OAuth2(cfg) => {
            match valid_access_token(conn, http, service.name(), &cfg, user_id).await {
                Ok(t) => Some(t),
                Err(ServiceError::NotConnected) => {
                    return Err(http_err(StatusCode::NOT_FOUND, "Not connected to service"))
                }
                Err(e) => return Err(http_err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())),
            }
        }
    };
    Ok(ServiceCtx { user_id, access_token, http: http.clone() })
}

pub async fn valid_access_token(
    conn: &mut AsyncPgConnection,
    http: &reqwest::Client,
    name: &str,
    cfg: &OAuthConfig,
    user_id: Uuid,
) -> Result<String, ServiceError> {
    let row = find_oauth_connection(conn, user_id, name)
        .await
        .map_err(db_err)?
        .ok_or(ServiceError::NotConnected)?;

    if row.expires_at - Duration::seconds(60) > Utc::now() {
        return dec(&row.access_token);
    }

    let refresh_token = row.refresh_token.as_deref().map(dec).transpose()?
        .ok_or(ServiceError::NotConnected)?;
    let creds = ClientCreds::from_env(name)?;
    let fresh = oauth::refresh(http, cfg, &creds, &refresh_token).await?;

    tokens_update(
        conn,
        row.id,
        enc(&fresh.access_token)?,
        fresh.refresh_token.as_deref().map(enc).transpose()?,
        fresh.expires_at(),
    )
    .await
    .map_err(db_err)?;

    Ok(fresh.access_token)
}
