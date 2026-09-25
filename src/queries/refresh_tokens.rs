use chrono::{Utc};
use sqlx::{query, query_as, PgConnection};
use uuid::Uuid;

use crate::{
    structs::db::{RefreshToken},
    password::{hash_token}
};

pub async fn insert_refresh_token(
    conn: &mut PgConnection,
    user_id: Uuid,
    secret: String
) -> Result<RefreshToken, sqlx::Error> {
    let refresh_token = RefreshToken {
        id: Uuid::new_v4(),
        user_id: user_id.clone(),
        token_hash: hash_token(&secret),
        created_at: Utc::now()
    };

    query!(
        r#"INSERT INTO refresh_tokens (id, user_id, token_hash, created_at)
           VALUES ($1, $2, $3, $4)"#,
        refresh_token.id,
        user_id,
        refresh_token.token_hash,
        refresh_token.created_at
    )
    .execute(conn)
    .await?;
    Ok(refresh_token)
}

pub async fn find_refresh_token(
    conn: &mut PgConnection,
    token_id: Uuid,
) -> Result<Option<RefreshToken>, sqlx::Error> {
    query_as!(
        RefreshToken,
        r#"SELECT * FROM refresh_tokens WHERE id = $1"#,
        token_id
    )
    .fetch_optional(conn)
    .await
}

pub async fn delete_refresh_token(conn: &mut PgConnection, token_id: Uuid) -> Result<(), sqlx::Error> {
    query!(r#"DELETE FROM refresh_tokens WHERE id = $1"#, token_id)
        .execute(conn)
        .await?;
    Ok(())
}
