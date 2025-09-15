use thiserror::Error;

use crate::core::{
    auth::hasher::hash_password,
    db::repos::{Pool, is_unique_constraint_violation, users_repo::User},
    validators::validate_length,
};

#[derive(Error, Debug)]
pub enum RegisterError {
    #[error("Invalid username")]
    InvalidUsername,
    #[error("Invalid password")]
    InvalidPassword,
    #[error("Username already exists")]
    UsernameAlreadyExists,
    #[error("Internal server error")]
    InternalServerError,
}

/// Registers a new user with the given username and password.
pub async fn register(pool: &Pool, username: &str, password: &str) -> Result<(), RegisterError> {
    validate_length(username, 1, 250).map_err(|_| RegisterError::InvalidUsername)?;
    validate_length(password, 8, 100).map_err(|_| RegisterError::InvalidPassword)?;

    let hashed_password = hash_password(password);
    let insertion_result = User::insert(pool, username, &hashed_password).await;

    match insertion_result {
        Ok(_) => Ok(()),
        Err(err) => {
            if is_unique_constraint_violation(&err) {
                Err(RegisterError::UsernameAlreadyExists)
            } else {
                Err(RegisterError::InternalServerError)
            }
        }
    }
}
