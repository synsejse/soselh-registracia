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

use db::VotingDB;
use routes::voting;

#[rocket::launch]
fn rocket() -> _ {
    let mut figment = rocket::config::Config::figment();

    // Allow setting database URL via environment variable
    if let Ok(database_url) = std::env::var("DATABASE_URL") {
        figment = figment.merge((
            "databases.voting_db",
            rocket_db_pools::Config {
                url: database_url,
                min_connections: None,
                max_connections: 1024,
                connect_timeout: 3,
                idle_timeout: None,
                extensions: None,
            },
        ));
    }

    rocket::custom(figment)
        .attach(VotingDB::init())
        .attach(AdHoc::on_ignite("Database Migrations", db::run_migrations))
        .attach(AdHoc::on_ignite("Database Seeding", db::run_seeding))
        .mount(
            "/api",
            routes![
                voting::client::create_session,
                voting::client::get_session_info,
                voting::client::get_candidates,
                voting::client::cast_vote,
                voting::client::get_vote_status,
                voting::admin::set_voting_status,
                voting::admin::get_stats,
                voting::admin::get_results,
                voting::admin::pick_winner
            ],
        )
        .mount("/", FileServer::from("/app/static"))
        .register("/", catchers![routes::not_found])
}
