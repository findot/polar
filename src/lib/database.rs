use rocket_sync_db_pools::database;
use rocket_sync_db_pools::diesel::PgConnection;

#[database("postgresql_pool")]
pub struct DbConnection(PgConnection);
