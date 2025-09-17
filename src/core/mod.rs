pub mod accounts;
pub mod auth;
mod db;
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

    state::CoreState { pool }
}
