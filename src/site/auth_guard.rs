use rocket::{
    Request,
    http::Status,
    request::{FromRequest, Outcome},
};

use crate::core::{
    UserId,
    auth::{ValidateSessionTokenError, validate_session_token},
    state::CoreState,
};

/// Represents a session token passed through an HTTP cookie. Implements `FromRequest` to be able to
/// use it as a request guard, which checks that the passed token is registered in the sessions table
/// and is not expired.
pub struct CookieAuthenticatedUser<'r> {
    /// The ID of the user associated with the session.
    pub user_id: UserId,

    /// The token used to authenticate the user.
    pub token: &'r str,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for CookieAuthenticatedUser<'r> {
    type Error = ValidateSessionTokenError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db_state = request.rocket().state::<CoreState>().unwrap();

        let cookies = request.cookies();
        match cookies.get("session_token") {
            Some(cookie) => {
                let token = cookie.value();
                let session_result = validate_session_token(&db_state.pool, token).await;

                match session_result {
                    Ok(session) => Outcome::Success(CookieAuthenticatedUser {
                        user_id: session.user_id,
                        token,
                    }),
                    Err(_) => Outcome::Forward(Status::Unauthorized),
                }
            }
            None => Outcome::Forward(Status::Unauthorized),
        }
    }
}
