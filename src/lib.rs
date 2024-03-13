use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};

use database::Database;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::SqliteConnection;

use user::{forgot, register};

use std::error::Error;

mod database;
mod models;
mod schema;
mod user;

const PORT: u16 = 8080;
type DatabasePool = Pool<ConnectionManager<SqliteConnection>>;

pub async fn run() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    env_logger::init();

    let database = Database::new()?;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(database.clone()))
            .service(register)
            .service(forgot)
            .wrap(Logger::default())
    })
    .bind(("0.0.0.0", PORT))?
    .workers(4)
    .run()
    .await?;

    Ok(())
}
