// Database connection and initialization for registration system

use diesel::Connection;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use rocket::Rocket;
use rocket_db_pools::Database;
use rocket_db_pools::diesel::MysqlPool;
use crate::config::AppConfig;

/// Database connection pool for registration system
#[derive(Database)]
#[database("registration_db")]
pub struct RegistrationDB(MysqlPool);

// Embed migrations from the migrations directory
const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Run pending database migrations
pub async fn run_migrations(rocket: Rocket<rocket::Build>) -> Rocket<rocket::Build> {
    let database_url = rocket.state::<AppConfig>()
        .expect("AppConfig not managed")
        .database_url
        .clone();

    // Run migrations in a blocking task since MigrationHarness requires sync connection
    let result: Result<Vec<String>, String> = rocket::tokio::task::spawn_blocking(move || {
        // Establish a new synchronous connection for migrations
        let mut sync_conn = diesel::MysqlConnection::establish(&database_url)
            .map_err(|e| format!("Failed to establish connection: {}", e))?;

        // Run migrations
        let versions = sync_conn
            .run_pending_migrations(MIGRATIONS)
            .map_err(|e| format!("Failed to run migrations: {}", e))?
            .into_iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>();

        Ok(versions)
    })
    .await
    .expect("Migration task panicked");

    match result {
        Ok(versions) => {
            if versions.is_empty() {
                println!("✅ Database is up to date");
            } else {
                println!("✅ Applied {} migration(s):", versions.len());
                for version in versions {
                    println!("   - {}", version);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ {}", e);
            panic!("Database migration failed");
        }
    }

    rocket
}
