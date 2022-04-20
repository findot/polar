use rocket::figment::{Error as FigmentError, Figment};
use rocket_sync_db_pools::database;
use rocket_sync_db_pools::diesel::PgConnection;
use std::collections::HashMap;

use crate::lib::config::Config;

#[database("postgresql_pool")]
pub struct DbConnection(PgConnection);

pub fn with_pool(figment: Figment) -> Result<Figment, FigmentError> {
    figment
        .extract()
        .map(|config: Config| {
            HashMap::from([(
                "postgresql_pool",
                HashMap::from([("url", format!("{}", config.database))]),
            )])
        })
        .map(|pool| Figment::from(("databases", pool)))
        .map(|db_figment| figment.merge(db_figment))
}
