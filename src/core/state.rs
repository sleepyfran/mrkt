use sqlx::SqlitePool;

/// Represents the managed state of Rocket for the database connection pool.
pub struct CoreState {
    pub pool: SqlitePool,
}
