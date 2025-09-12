use sqlx::SqlitePool;

/// Represents the managed state of Rocket for the database connection pool.
pub struct DbState {
    pub pool: SqlitePool,
}
