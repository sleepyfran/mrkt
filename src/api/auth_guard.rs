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
pub struct SessionToken<'r>(&'r str);

#[derive(Debug)]
pub enum SessionTokenError {
    InvalidToken,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for SessionToken<'r> {
    type Error = SessionTokenError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        async fn is_valid_key(bearer: &str) -> bool {
            let mut connection = create_connection();
            let key = bearer.strip_prefix("Bearer ");
            if let None = key {
                return false;
            }

            let key = key.unwrap();
            let session = sessions::table
                .filter(sessions::token.eq(key))
                .select(Session::as_select())
                .first::<Session>(&mut connection);

            match session {
                Ok(session) => {
                    let current_timestamp = Utc::now().naive_utc();
                    session.expires_at.unwrap() > current_timestamp
                }
                _ => false,
            }
        }

        match request.headers().get_one("Authorization") {
            Some(token) if is_valid_key(token).await => Outcome::Success(SessionToken(token)),
            _ => Outcome::Error((Status::Unauthorized, SessionTokenError::InvalidToken)),
        }
    }
}
