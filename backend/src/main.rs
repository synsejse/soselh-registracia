// Main application entry point

#[macro_use]
extern crate rocket;

mod db;
mod models;
mod routes;
mod schema;

use diesel::prelude::*;
use rocket::fairing::AdHoc;
use rocket::fs::FileServer;
use rocket_db_pools::Database;
use std::sync::atomic::AtomicBool;
use tokio::sync::broadcast;

use db::VotingDB;
use routes::voting;

pub struct AppState {
    pub voting_enabled: AtomicBool,
    pub tx: broadcast::Sender<bool>,
}

async fn load_initial_state(
    rocket: rocket::Rocket<rocket::Build>,
) -> rocket::Rocket<rocket::Build> {
    let enabled = rocket::tokio::task::spawn_blocking(|| {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let mut conn = diesel::MysqlConnection::establish(&database_url)
            .expect("Failed to connect to DB for state loading");

        use schema::settings::dsl::*;

        settings
            .find("voting_enabled")
            .select(value)
            .first::<String>(&mut conn)
            .map(|v| v == "true")
            .unwrap_or(false)
    })
    .await
    .expect("State loading task failed");

    let (tx, _) = broadcast::channel(100);

    rocket.manage(AppState {
        voting_enabled: AtomicBool::new(enabled),
        tx,
    })
}

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
        .attach(AdHoc::on_ignite("Load Initial State", load_initial_state))
        .mount(
            "/api",
            routes![
                voting::client::create_session,
                voting::client::get_session_info,
                voting::client::get_candidates,
                voting::client::cast_vote,
                voting::client::voting_status_ws,
                voting::admin::set_voting_status,
                voting::admin::get_stats,
                voting::admin::get_results,
                voting::admin::pick_winner
            ],
        )
        .mount("/", FileServer::from("/app/static"))
        .register("/", catchers![routes::not_found])
}
