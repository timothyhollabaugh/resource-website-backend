#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

pub mod access;
pub mod chemicals;
pub mod errors;
pub mod search;
pub mod users;

use log::info;
use log::warn;

use diesel::r2d2::ConnectionManager;
use diesel::MysqlConnection;
use r2d2::Pool;

use errors::Error;
use errors::ErrorKind;

embed_migrations!();

pub fn init_database(
    url: &str,
) -> Result<Pool<ConnectionManager<MysqlConnection>>, Error> {
    info!("Connecting to database");
    let manager = ConnectionManager::new(url);

    let pool = Pool::builder().max_size(15).build(manager);

    let pool = match pool {
        Ok(p) => p,
        Err(e) => {
            return Err(Error::with_source(ErrorKind::Database, Box::new(e)))
        }
    };

    info!("Running migrations");
    if let Err(e) = embedded_migrations::run(&pool.get()?) {
        warn!("Could not run migrations: {}", e);
    }

    Ok(pool)
}

pub enum HttpMethod {
    GET,
    POST,
    DELETE,
}

