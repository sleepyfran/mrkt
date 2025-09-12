pub mod repos;
pub mod state;

use dotenvy::dotenv;
use sqlx::SqlitePool;

/// Creates and returns a new SQLite connection to the database specified by the environment
/// variable DATABASE_URL.
pub async fn create_pool() -> SqlitePool {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqlitePool::connect(&database_url)
        .await
        .expect(&format!("Error connecting to {}", database_url))
}
