use crate::config::DatabaseConfig;
use crate::result::DatabaseError;
use diesel::{pg::PgConnection, Connection, ConnectionResult};
use diesel_migrations::embed_migrations;
use rocket_sync_db_pools::database;

#[database("postgresql_pool")]
pub struct DbConnection(PgConnection);

pub fn establish_connection(db_config: &DatabaseConfig) -> ConnectionResult<PgConnection> {
    let url = db_config.to_string();
    PgConnection::establish(&url)
}

embed_migrations!("./resources/migrations/postgres");

pub fn migrate(db_config: &DatabaseConfig, output: bool) -> Result<(), DatabaseError> {
    let conn = establish_connection(db_config)?;
    if output {
        embedded_migrations::run_with_output(&conn, &mut std::io::stderr())?;
    } else {
        embedded_migrations::run(&conn)?;
    }
    Ok(())
}
