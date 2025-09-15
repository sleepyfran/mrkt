use crate::core::db::repos::{
    DatabaseError, Pool,
    sessions_repo::{Session, Token},
    users_repo::UserId,
};
use argon2::password_hash::rand_core::{OsRng, RngCore};
use base64::{Engine, prelude::BASE64_STANDARD};
use thiserror::Error;
use time::OffsetDateTime;

/// Generates a 32-byte random token and returns it as a base64-encoded string.
pub fn generate_session_token() -> String {
    let mut token_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut token_bytes);
    BASE64_STANDARD.encode(token_bytes)
}

/// Creates a new session for the given user ID, generating a unique session token, saving it
/// to the database, and returning the session token that was created.
pub async fn create_session(pool: &Pool, user_id: UserId) -> Result<Token, DatabaseError> {
    let session_token = generate_session_token();

    let tx = pool.begin().await?;

    let _ = Session::cleanup_expired(pool).await;
    let session = Session::insert(pool, user_id, session_token).await?;

    let _ = tx.commit().await;

    Ok(session.token)
}

#[derive(Error, Debug)]
pub enum ValidateSessionTokenError {
    #[error("The token has expired")]
    ExpiredToken,
    #[error("Session token does not exist or is invalid")]
    NonExistentToken(String),
    #[error("Database error")]
    DatabaseError(#[from] DatabaseError),
}

pub async fn validate_session_token(
    pool: &Pool,
    token: &str,
) -> Result<Session, ValidateSessionTokenError> {
    let session = Session::by_token(pool, token).await?;

    match session {
        Some(session) => {
            let current_timestamp = OffsetDateTime::now_utc();
            let is_valid_session = session.expires_at.unwrap() > current_timestamp;

            if is_valid_session {
                Ok(session)
            } else {
                Err(ValidateSessionTokenError::ExpiredToken)
            }
        }
        None => Err(ValidateSessionTokenError::NonExistentToken(
            token.to_string(),
        )),
    }
}
