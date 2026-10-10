use diesel::{prelude::*, result::Error, dsl::exists};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::{
    schema::oauth_connections,
    structs::{
        db::OauthConnections,
        oauth::NewOauthConnection,
    },
};

pub async fn insert_oauth_connection(
    conn: &mut AsyncPgConnection,
    new: NewOauthConnection
) -> Result<OauthConnections, Error> {
    let oauth_connection = OauthConnections {
        id: Uuid::new_v4(),
        user_id: new.user_id,
        provider: new.provider,
        provider_user_id: new.provider_user_id,
        access_token: new.access_token,
        refresh_token: new.refresh_token,
        expires_at: new.expires_at,
        metadata: new.metadata
    };

    diesel::insert_into(oauth_connections::table)
        .values(&oauth_connection)
        .execute(conn)
        .await?;
    Ok(oauth_connection)
}

pub async fn find_oauth_connection(
    conn: &mut AsyncPgConnection,
    uid: Uuid,
    provider: &str,
) -> Result<Option<OauthConnections>, Error> {
    oauth_connections::table
        .filter(oauth_connections::user_id.eq(uid))
        .filter(oauth_connections::provider.eq(provider))
        .select(OauthConnections::as_select())
        .first(conn)
        .await
        .optional()
}

pub async fn oauth_connection_exists(
    conn: &mut AsyncPgConnection,
    uid: Uuid,
    provider: &str,
) -> Result<bool, Error> {
    diesel::select(exists(
        oauth_connections::table
            .filter(oauth_connections::user_id.eq(uid))
            .filter(oauth_connections::provider.eq(provider)),
    ))
    .get_result(conn)
    .await
}

pub async fn tokens_update(
    conn: &mut AsyncPgConnection,
    conn_id: Uuid,
    access_token: String,
    refresh_token: Option<String>,
    expires_at: DateTime<Utc>,
) -> Result<(), Error> {
    let target = diesel::update(oauth_connections::table.find(conn_id));
    match refresh_token {
        Some(rt) => target
            .set((
                oauth_connections::access_token.eq(access_token),
                oauth_connections::refresh_token.eq(rt),
                oauth_connections::expires_at.eq(expires_at),
            ))
            .execute(conn)
            .await?,
        None => target
            .set((
                oauth_connections::access_token.eq(access_token),
                oauth_connections::expires_at.eq(expires_at),
            ))
            .execute(conn)
            .await?,
    };
    Ok(())
}

pub async fn delete_oauth_connection(
    conn: &mut AsyncPgConnection,
    uid: Uuid,
    provider: &str,
) -> Result<(), Error> {
    diesel::delete(
        oauth_connections::table
            .filter(oauth_connections::user_id.eq(uid))
            .filter(oauth_connections::provider.eq(provider)),
    )
    .execute(conn)
    .await?;
    Ok(())
}
