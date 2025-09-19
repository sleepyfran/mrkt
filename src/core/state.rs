use std::sync::Arc;

use sqlx::SqlitePool;

use crate::core::data_sources::MarketProvider;

/// Represents the managed state of Rocket for the database connection pool.
pub struct CoreState {
    /// Pool of database connections.
    pub pool: SqlitePool,

    /// Chosen market data provider (e.g., "AlphaVantage").
    pub market_provider: Arc<dyn MarketProvider>,
}
