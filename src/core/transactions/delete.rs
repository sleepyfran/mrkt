use thiserror::Error;

use crate::core::{
    Transaction, TransactionId, UserId,
    db::repos::{DatabaseError, Pool},
};

#[derive(Error, Debug)]
pub enum DeleteTransactionError {
    #[error("Transaction not found")]
    TransactionNotFound,
    #[error("Database error")]
    DatabaseError(#[from] DatabaseError),
}

/// Attempts to delete a transaction with the given ID for the specified user ID.
/// Only allows users to delete their own transactions.
pub async fn delete_transaction(
    pool: &Pool,
    belonging_to_user_id: UserId,
    transaction_id: TransactionId,
) -> Result<(), DeleteTransactionError> {
    let transaction = Transaction::by_id(pool, transaction_id, belonging_to_user_id).await?;

    if transaction.is_none() {
        return Err(DeleteTransactionError::TransactionNotFound);
    }

    Transaction::delete(pool, transaction_id).await?;

    Ok(())
}
