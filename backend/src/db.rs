// Database connection and initialization

use diesel::Connection;
use diesel::prelude::*;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use rocket::Rocket;
use rocket_db_pools::Database;
use rocket_db_pools::diesel::MysqlPool;

/// Database connection pool for voting
#[derive(Database)]
#[database("voting_db")]
pub struct VotingDB(MysqlPool);

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

/// Seed database with initial data
pub async fn run_seeding(rocket: Rocket<rocket::Build>) -> Rocket<rocket::Build> {
    let result: Result<(), String> = rocket::tokio::task::spawn_blocking(move || {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let mut sync_conn = diesel::MysqlConnection::establish(&database_url)
            .map_err(|e| format!("Failed to establish connection: {}", e))?;

        if let Ok(candidates_env) = std::env::var("CANDIDATES") {
            use crate::schema::candidates::dsl::*;

            let count: i64 = candidates.count().get_result(&mut sync_conn).unwrap_or(0);

            if count == 0 {
                let new_candidates: Vec<crate::models::NewCandidate> = candidates_env
                    .split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .map(|s| crate::models::NewCandidate {
                        name: s.to_string(),
                    })
                    .collect();

                if !new_candidates.is_empty() {
                    diesel::insert_into(candidates)
                        .values(&new_candidates)
                        .execute(&mut sync_conn)
                        .map_err(|e| format!("Failed to seed candidates: {}", e))?;
                    println!(
                        "🌱 Seeded {} candidates from environment variable",
                        new_candidates.len()
                    );
                }
            }
        }
        Ok(())
    })
    .await
    .expect("Seeding task panicked");

    if let Err(e) = result {
        eprintln!("❌ Seeding failed: {}", e);
    }

    rocket
}
