use chrono::{Utc};
use sqlx::{query, query_as, PgConnection};
use uuid::Uuid;

use crate::structs::db::{User};

pub async fn insert_user(
    conn: &mut PgConnection,
    mail: String,
    name: String,
    password_hash: String
) -> Result<User, sqlx::Error> {
    let user = User {
        id: Uuid::new_v4(),
        mail: mail.clone(),
        name: name.clone(),
        password_hash: password_hash.clone(),
        dashboard_map: None,
        created_at: Utc::now()
    };

    query!(
        r#"INSERT INTO users (id, mail, name, password_hash, created_at)
           VALUES ($1, $2, $3, $4, $5)"#,
        user.id,
        mail,
        name,
        password_hash,
        user.created_at
    )
    .execute(conn)
    .await?;
    Ok(user)
}

pub async fn find_user_by_mail(
    conn: &mut PgConnection,
    mail: String,
) -> Result<Option<User>, sqlx::Error> {
    query_as!(User, r#"SELECT * FROM users WHERE mail = $1 LIMIT 1"#, mail)
        .fetch_optional(conn)
        .await
}

pub async fn user_exists(conn: &mut PgConnection, id: Uuid) -> Result<bool, sqlx::Error> {
    let found = query_as!(
        User,
        r#"SELECT * FROM users WHERE id = $1 LIMIT 1"#,
        id
    )
    .fetch_optional(conn)
    .await?;
    Ok(found.is_some())
}
