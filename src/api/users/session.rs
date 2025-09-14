use crate::{
    core::auth::generate_session_token,
    db::repos::{
        DatabaseError, Pool,
        sessions_repo::{Session, Token},
        users_repo::UserId,
    },
};

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
