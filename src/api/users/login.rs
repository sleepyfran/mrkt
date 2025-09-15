use rocket::State;
use rocket::http::Status;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use crate::core::{
    auth::{LoginError, login as _login},
    state::CoreState,
};
use crate::impl_responder;

impl_responder! {
    LoginError {
        LoginError::InvalidCredentials => Status::Unauthorized,
        LoginError::DatabaseError(_) => Status::InternalServerError,
    }
}

#[post("/users/login", data = "<login_data>")]
pub async fn login(
    db_state: &State<CoreState>,
    login_data: Json<LoginData>,
) -> Result<Json<LoginResponse>, LoginError> {
    let token = _login(&db_state.pool, &login_data.username, &login_data.password).await?;
    Ok(Json(LoginResponse { token: token }))
}

/// Represents a user attempting to login.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct LoginData {
    /// The username of the user.
    pub username: String,
    /// The unhashed password of the user.
    pub password: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct LoginResponse {
    /// The generated session token for the user that is used to authenticate requests.
    pub token: String,
}
