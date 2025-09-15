use thiserror::Error;

use crate::core::{
    Account, UserId,
    db::repos::{DatabaseError, Pool},
    validators::validate_length,
};

#[derive(Error, Debug)]
pub enum CreateAccountError {
    #[error("Invalid name")]
    InvalidName,
    #[error("Database error")]
    DatabaseError(#[from] DatabaseError),
}

/// Attempts to create a new account with the given name for the specified user ID.
pub async fn create_account(
    pool: &Pool,
    belonging_to_user_id: UserId,
    name: &str,
) -> Result<Account, CreateAccountError> {
    validate_length(name, 1, 250).map_err(|_| CreateAccountError::InvalidName)?;
    let account = Account::insert(pool, name, belonging_to_user_id).await?;
    Ok(account)
}
