use thiserror::Error;

use crate::core::{
    Transaction, UserId,
    db::repos::{DatabaseError, Pool},
};

#[derive(Error, Debug)]
pub enum ListAllError {
    #[error("Database error")]
    DatabaseError(#[from] DatabaseError),
}

/// Retrieves all transactions belonging to the specified user ID.
pub async fn list_all(
    pool: &Pool,
    belonging_to_user_id: UserId,
) -> Result<Vec<Transaction>, ListAllError> {
    let transactions = Transaction::get_all(pool, belonging_to_user_id).await?;
    Ok(transactions)
}
