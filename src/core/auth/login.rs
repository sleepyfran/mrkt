use thiserror::Error;

use crate::core::{
    auth::{hasher::verify_password, session::create_session},
    db::repos::{DatabaseError, Pool, sessions_repo::Token, users_repo::User},
};

#[derive(Error, Debug)]
pub enum LoginError {
    #[error("Invalid username or password")]
    InvalidCredentials,
    #[error("Internal server error")]
    DatabaseError(#[from] DatabaseError),
}

/// Logs in a user with the given username and password, returning a session
/// token if successful.
pub async fn login(pool: &Pool, username: &str, password: &str) -> Result<Token, LoginError> {
    let db_user = User::by_username(pool, username).await?;

    if let Some(db_user) = db_user {
        let password_is_correct = verify_password(password, &db_user.hashed_password);
        if !password_is_correct {
            return Err(LoginError::InvalidCredentials);
        }

        // TODO: Potentially limit the number of sessions.
        let token = create_session(pool, db_user.id.unwrap()).await?;
        Ok(token)
    } else {
        Err(LoginError::InvalidCredentials)
    }
}

/// Checks if there are any users registered in the system.
pub async fn has_any_user(pool: &Pool) -> bool {
    match User::count(pool).await {
        Ok(count) => count > 0,
        Err(_) => false, // Return false on error to avoid blocking access.
    }
}
