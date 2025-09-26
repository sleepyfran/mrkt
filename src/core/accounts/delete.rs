use thiserror::Error;

use crate::core::{
    Account, AccountId, UserId,
    db::repos::{DatabaseError, Pool},
};

#[derive(Error, Debug)]
pub enum DeleteAccountError {
    #[error("Account not found")]
    AccountNotFound,
    #[error("Database error")]
    DatabaseError(#[from] DatabaseError),
}

/// Attempts to delete an account with the given ID for the specified user ID.
/// Only allows users to delete their own accounts.
pub async fn delete_account(
    pool: &Pool,
    belonging_to_user_id: UserId,
    account_id: AccountId,
) -> Result<(), DeleteAccountError> {
    let account = Account::by_id(pool, account_id).await?;

    if let Some(account) = account {
        if account.owner_id != belonging_to_user_id {
            return Err(DeleteAccountError::AccountNotFound);
        }
    } else {
        return Err(DeleteAccountError::AccountNotFound);
    }

    Account::delete(pool, account_id).await?;

    Ok(())
}
