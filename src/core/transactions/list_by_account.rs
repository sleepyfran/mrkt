use thiserror::Error;

use crate::core::{
    AccountId, Transaction, UserId,
    db::repos::{DatabaseError, Pool},
};

#[derive(Error, Debug)]
pub enum ListByAccountError {
    #[error("Database error")]
    DatabaseError(#[from] DatabaseError),
}

/// Retrieves all transactions belonging to the specified user ID and account ID.
pub async fn list_by_account(
    pool: &Pool,
    belonging_to_user_id: UserId,
    account_id: AccountId,
) -> Result<Vec<Transaction>, ListByAccountError> {
    let transactions = Transaction::get_by_account(pool, belonging_to_user_id, account_id).await?;
    Ok(transactions)
}