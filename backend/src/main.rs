// Main application entry point - School Registration System

#[macro_use]
extern crate rocket;

mod db;
mod models;
mod routes;
mod schema;
mod config;

use config::AppConfig;
use diesel::prelude::*;
use rocket::fairing::AdHoc;
use rocket::fs::FileServer;
use rocket_db_pools::Database;
use std::sync::atomic::AtomicBool;
use tokio::sync::broadcast;

use db::RegistrationDB;
use routes::registration;

pub struct AppState {
    pub registration_enabled: AtomicBool,
    pub tx: broadcast::Sender<bool>,
    pub admin_password_hash: String,
}

async fn load_initial_state(
    rocket: rocket::Rocket<rocket::Build>,
) -> rocket::Rocket<rocket::Build> {
    let config = rocket.state::<AppConfig>().expect("AppConfig not managed").clone();

    let enabled = rocket::tokio::task::spawn_blocking(move || {
        let mut conn = diesel::MysqlConnection::establish(&config.database_url)
            .expect("Failed to connect to DB for state loading");

        use schema::settings::dsl::*;

        settings
            .find("registration_enabled")
            .select(value)
            .first::<String>(&mut conn)
            .map(|v| v == "true")
            .unwrap_or(false)
    })
    .await
    .expect("State loading task failed");

    let (tx, _) = broadcast::channel(100);

    let admin_password_hash = config.admin_password_hash.clone();

    rocket.manage(AppState {
        registration_enabled: AtomicBool::new(enabled),
        tx,
        admin_password_hash,
    })
}

#[rocket::launch]
fn rocket() -> rocket::Rocket<rocket::Build> {
    dotenvy::dotenv().ok();

    let config = AppConfig::load();
    let mut figment = rocket::config::Config::figment()
        .merge(("port", config.rocket_port))
        .merge(("address", &config.rocket_address));

    figment = figment.merge((
        "databases.registration_db",
        rocket_db_pools::Config {
            url: config.database_url.clone(),
            min_connections: Some(1),
            max_connections: 1024,
            connect_timeout: 3,
            idle_timeout: None,
            extensions: None,
        },
    ));

    rocket::custom(figment)
        .manage(config.clone())
        .attach(RegistrationDB::init())
        .attach(AdHoc::on_ignite("Database Migrations", db::run_migrations))
        .attach(AdHoc::on_ignite("Load Initial State", load_initial_state))
        .mount(
            "/api",
            routes![
                registration::client::get_sessions,
                registration::client::create_registration,
                registration::client::get_registration_status,
                registration::admin::admin_login,
                registration::admin::admin_logout,
                registration::admin::admin_check,
                registration::admin::get_all_registrations,
                registration::admin::toggle_registration,
                registration::admin::export_registrations_excel,
                registration::admin::confirm_registration,
                registration::admin::delete_registration,
            ],
        )
        .mount("/", FileServer::from(&config.static_dir))
        .register("/", catchers![routes::not_found, routes::unauthorized])
}
