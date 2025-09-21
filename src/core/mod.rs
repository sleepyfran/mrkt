pub mod accounts;
pub mod auth;
mod cache;
pub mod data_sources;
mod db;
mod logger;
pub mod portfolio;
pub mod shared;
pub mod state;
pub mod transactions;
pub mod validators;

pub use db::repos::{
    accounts_repo::{Account, AccountId},
    transactions_repo::{Transaction, TransactionId, TransactionType},
    users_repo::UserId,
};

/// Initializes the core application state, which creates a database connection
/// pool, runs migrations, and returns a state that can be managed by Rocket
/// to provide database access throughout the application.
pub async fn init() -> state::CoreState {
    // Start by preparing the database.
    let pool = db::create_pool().await;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to migrate database");

    // Next, set up the market data provider.
    let market_provider = data_sources::create_market_provider();

    // Set up the exchange rate provider.
    let exchange_rate_provider = data_sources::create_exchange_rate_provider();

    // Setup the global logger.
    logger::setup_logger().expect("Failed to set up logger");

    logger::info!("Finished setting up core state");

    state::CoreState {
        pool,
        market_provider,
        exchange_rate_provider,
    }
}
