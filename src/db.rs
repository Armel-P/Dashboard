use anyhow::anyhow;
use diesel::Connection;
use diesel_async::{
    AsyncPgConnection,
    async_connection_wrapper::AsyncConnectionWrapper,
    pooled_connection::{
        AsyncDieselConnectionManager,
        deadpool::{Object, Pool},
    },
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

pub type DbPool = Pool<AsyncPgConnection>;
pub type DbConn = Object<AsyncPgConnection>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn connect(database_url: &str) -> anyhow::Result<DbPool> {
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    Pool::builder(manager)
        .max_size(5)
        .build()
        .map_err(|e| anyhow!("Error building DB pool: {e}"))
}

pub async fn run_migrations(database_url: &str) -> anyhow::Result<()> {
    let url = database_url.to_owned();
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let mut conn = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&url)?;
        conn.run_pending_migrations(MIGRATIONS)
            .map_err(|e| anyhow!("Migration error: {e}"))?;
        Ok(())
    })
    .await??;
    Ok(())
}
