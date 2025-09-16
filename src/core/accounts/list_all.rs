use thiserror::Error;

use crate::core::{
    Account, UserId,
    db::repos::{DatabaseError, Pool},
};

#[derive(Error, Debug)]
pub enum ListAllError {
    #[error("Database error")]
    DatabaseError(#[from] DatabaseError),
}

/// Retrieves all accounts belonging to the specified user ID.
pub async fn list_all(
    pool: &Pool,
    belonging_to_user_id: UserId,
) -> Result<Vec<Account>, ListAllError> {
    let accounts = Account::for_user(pool, belonging_to_user_id).await?;
    Ok(accounts)
}
