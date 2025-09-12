use chrono::Utc;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use rocket::{
    Request,
    http::Status,
    request::{FromRequest, Outcome},
};

use crate::db::{create_connection, model::session::Session, schema::sessions};

/// Represents a session token passed through an HTTP header. Implements `FromRequest` to be able to
/// use it as a request guard, which checks that the passed token is registered in the sessions table
/// and is not expired.
pub struct AuthenticatedUser<'r> {
    /// The ID of the user associated with the session.
    pub user_id: i32,

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
        async fn retrieve_valid_session(bearer: &str) -> Option<AuthenticatedUser> {
            let mut connection = create_connection();
            let key = bearer.strip_prefix("Bearer ");
            if let None = key {
                return None;
            }

            let key = key.unwrap();
            let session = sessions::table
                .filter(sessions::token.eq(key))
                .select(Session::as_select())
                .first::<Session>(&mut connection);

            match session {
                Ok(session) => {
                    let current_timestamp = Utc::now().naive_utc();
                    let is_valid_session = session.expires_at.unwrap() > current_timestamp;

                    if is_valid_session {
                        Some(AuthenticatedUser {
                            user_id: session.user_id,
                            token: key,
                        })
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }

        match request.headers().get_one("Authorization") {
            Some(token) => {
                let auth_user = retrieve_valid_session(token).await;
                match auth_user {
                    Some(user) => Outcome::Success(user),
                    None => Outcome::Error((Status::Unauthorized, SessionTokenError::InvalidToken)),
                }
            }
            _ => Outcome::Error((Status::Unauthorized, SessionTokenError::InvalidToken)),
        }
    }
}
