use chrono::Utc;
use diesel::{dsl::exists, prelude::*, result::Error};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    schema::users,
    structs::{db::User, records::MapRecord},
};

pub async fn insert_user(
    conn: &mut AsyncPgConnection,
    mail: String,
    name: String,
    password_hash: String,
) -> Result<User, Error> {
    let user = User {
        id: Uuid::new_v4(),
        mail,
        name,
        password_hash,
        dashboard_map: None,
        created_at: Utc::now(),
    };

    diesel::insert_into(users::table)
        .values(&user)
        .execute(conn)
        .await?;
    Ok(user)
}

pub async fn map_update(conn: &mut AsyncPgConnection, id: Uuid, map: Value) -> Result<(), Error> {
    diesel::update(users::table.find(id))
        .set(users::dashboard_map.eq(map))
        .execute(conn)
        .await?;
    Ok(())
}

pub async fn map_get(conn: &mut AsyncPgConnection, id: Uuid) -> Result<Option<MapRecord>, Error> {
    users::table
        .find(id)
        .select(MapRecord::as_select())
        .first(conn)
        .await
        .optional()
}

pub async fn delete_user(conn: &mut AsyncPgConnection, id: Uuid) -> Result<(), Error> {
    diesel::delete(users::table.find(id)).execute(conn).await?;
    Ok(())
}

pub async fn find_user_by_id(conn: &mut AsyncPgConnection, id: Uuid) -> Result<Option<User>, Error> {
    users::table
        .find(id)
        .select(User::as_select())
        .first(conn)
        .await
        .optional()
}

pub async fn find_user_by_mail(
    conn: &mut AsyncPgConnection,
    mail: String,
) -> Result<Option<User>, Error> {
    users::table
        .filter(users::mail.eq(mail))
        .select(User::as_select())
        .first(conn)
        .await
        .optional()
}

pub async fn user_exists(conn: &mut AsyncPgConnection, id: Uuid) -> Result<bool, Error> {
    diesel::select(exists(users::table.find(id)))
        .get_result(conn)
        .await
}
