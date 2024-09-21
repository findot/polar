use crate::config::DatabaseConfig;
use crate::result::DatabaseError;
use diesel::pg::PgConnection;
use diesel::Connection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use rocket_db_pools::diesel::PgPool;
use rocket_db_pools::Database;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./resources/migrations/postgres");

#[derive(Database)]
#[database("postgresql_pool")]
pub struct Db(PgPool);

pub fn establish_connection(db_config: &DatabaseConfig) -> Result<PgConnection, DatabaseError> {
    let url = db_config.to_string();
    let conn = PgConnection::establish(&url)?;
    Ok(conn)
}

pub fn migrate(db_config: &DatabaseConfig) -> Result<(), DatabaseError> {
    let mut conn = establish_connection(db_config)?;
    conn.run_pending_migrations(MIGRATIONS)?;
    Ok(())
}
