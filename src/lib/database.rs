use crate::config::DatabaseConfig;
use crate::result::DatabaseError;
use diesel::{pg::PgConnection, Connection, ConnectionResult};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use rocket_sync_db_pools::database;

#[database("postgresql_pool")]
pub struct DbConnection(PgConnection);

pub fn establish_connection(db_config: &DatabaseConfig) -> ConnectionResult<PgConnection> {
    let url = db_config.to_string();
    PgConnection::establish(&url)
}

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./resources/migrations/postgres");

pub fn migrate(db_config: &DatabaseConfig) -> Result<(), DatabaseError> {
    let mut conn = establish_connection(db_config)?;
    conn.run_pending_migrations(MIGRATIONS)?;
    Ok(())
}
