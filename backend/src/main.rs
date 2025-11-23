// Main application entry point

#[macro_use]
extern crate rocket;

mod db;
mod models;
mod routes;
mod schema;

use rocket::fairing::AdHoc;
use rocket::fs::FileServer;
use rocket_db_pools::Database;

use db::MessagesDB;
use routes::contact;

#[rocket::launch]
fn rocket() -> _ {
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");

    let figment = rocket::config::Config::figment().merge((
        "databases.messages_db",
        rocket_db_pools::Config {
            url: database_url,
            min_connections: None,
            max_connections: 1024,
            connect_timeout: 3,
            idle_timeout: None,
            extensions: None,
        },
    ));

    rocket::custom(figment)
        .attach(MessagesDB::init())
        .attach(AdHoc::on_ignite("Database Migrations", db::run_migrations))
        .mount("/", routes![contact::submit_message])
        .mount("/", FileServer::from("/app/static"))
        .register("/", catchers![routes::not_found])
}
