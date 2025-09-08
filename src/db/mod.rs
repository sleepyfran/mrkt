pub mod insertables;
pub mod model;
pub mod schema;

use diesel::prelude::*;
use dotenvy::dotenv;

/// Creates and returns a new SQLite connection to the database specified by the environment
/// variable DATABASE_URL.
///
/// TODO: Potentially move to have a pool of connections.
pub fn create_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}
