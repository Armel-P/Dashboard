use chrono::Utc;
use diesel::{prelude::*, result::Error};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    password::hash_token,
    schema::refresh_tokens,
    structs::db::RefreshToken};

pub async fn insert_refresh_token(
    conn: &mut AsyncPgConnection,
    user_id: Uuid,
    secret: String,
) -> Result<RefreshToken, Error> {
    let refresh_token = RefreshToken {
        id: Uuid::new_v4(),
        user_id,
        token_hash: hash_token(&secret),
        created_at: Utc::now(),
    };

    diesel::insert_into(refresh_tokens::table)
        .values(&refresh_token)
        .execute(conn)
        .await?;
    Ok(refresh_token)
}

pub async fn find_refresh_token(
    conn: &mut AsyncPgConnection,
    token_id: Uuid,
) -> Result<Option<RefreshToken>, Error> {
    refresh_tokens::table
        .find(token_id)
        .select(RefreshToken::as_select())
        .first(conn)
        .await
        .optional()
}

pub async fn delete_refresh_token(conn: &mut AsyncPgConnection, token_id: Uuid) -> Result<(), Error> {
    diesel::delete(refresh_tokens::table.find(token_id))
        .execute(conn)
        .await?;
    Ok(())
}
