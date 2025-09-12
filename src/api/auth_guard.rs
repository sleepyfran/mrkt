use rocket::{
    Request,
    http::Status,
    request::{FromRequest, Outcome},
};
use time::OffsetDateTime;

use crate::db::{
    repos::{sessions_repo::Session, users_repo::UserId},
    state::DbState,
};

/// Represents a session token passed through an HTTP header. Implements `FromRequest` to be able to
/// use it as a request guard, which checks that the passed token is registered in the sessions table
/// and is not expired.
pub struct AuthenticatedUser<'r> {
    /// The ID of the user associated with the session.
    pub user_id: UserId,

    /// The token used to authenticate the user.
    pub token: &'r str,
}

#[derive(Debug)]
pub enum SessionTokenError {
    InvalidToken,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser<'r> {
    type Error = SessionTokenError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db_state = request.rocket().state::<DbState>().unwrap();

        let error_outcome = Outcome::Error((Status::Unauthorized, SessionTokenError::InvalidToken));
        match request.headers().get_one("Authorization") {
            Some(token) => {
                let key = token.strip_prefix("Bearer ");
                if let None = key {
                    return error_outcome;
                }

                let key = key.unwrap();
                let session = Session::by_token(&db_state.pool, key).await;

                match session {
                    Ok(Some(session)) => {
                        let current_timestamp = OffsetDateTime::now_utc();
                        let is_valid_session = session.expires_at.unwrap() > current_timestamp;

                        if is_valid_session {
                            Outcome::Success(AuthenticatedUser {
                                user_id: session.user_id,
                                token: key,
                            })
                        } else {
                            error_outcome
                        }
                    }
                    _ => error_outcome,
                }
            }
            _ => error_outcome,
        }
    }
}
