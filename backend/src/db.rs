// Database connection and initialization

use diesel::Connection;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use rocket::Rocket;
use rocket_db_pools::Database;
use rocket_db_pools::diesel::MysqlPool;

/// Database connection pool for messages
#[derive(Database)]
#[database("messages_db")]
pub struct MessagesDB(MysqlPool);

// Embed migrations from the migrations directory
const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Run pending database migrations
pub async fn run_migrations(rocket: Rocket<rocket::Build>) -> Rocket<rocket::Build> {
    // Run migrations in a blocking task since MigrationHarness requires sync connection
    let result: Result<Vec<String>, String> = rocket::tokio::task::spawn_blocking(move || {
        // Establish a new synchronous connection for migrations
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let mut sync_conn = diesel::MysqlConnection::establish(&database_url)
            .map_err(|e| format!("Failed to establish connection: {}", e))?;

        sync_conn
            .run_pending_migrations(MIGRATIONS)
            .map(|versions| {
                versions
                    .into_iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
            })
            .map_err(|e| format!("Failed to run migrations: {}", e))
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
