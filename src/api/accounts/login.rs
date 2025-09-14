use rocket::State;
use rocket::http::Status;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::api::accounts::session::create_session;
use crate::core::auth::verify_password;
use crate::db::repos::Pool;
use crate::db::repos::users_repo::User;
use crate::db::state::DbState;
use crate::impl_responder;

#[derive(Error, Debug)]
pub enum LoginError {
    #[error("Invalid username or password")]
    InvalidCredentials,
    #[error("Internal server error")]
    ServerError,
}

impl_responder! {
    LoginError {
        LoginError::InvalidCredentials => Status::Unauthorized,
        LoginError::ServerError => Status::InternalServerError,
    }
}

#[post("/login", data = "<login_data>")]
pub async fn login(
    db_state: &State<DbState>,
    login_data: Json<LoginData>,
) -> Result<Json<LoginResponse>, LoginError> {
    let db_user = User::find_by_username(&db_state.pool, &login_data.username)
        .await
        .map_err(|_| LoginError::ServerError)?;

    if let Some(db_user) = db_user {
        let password_is_correct = verify_password(&login_data.password, &db_user.hashed_password);
        if !password_is_correct {
            return Err(LoginError::InvalidCredentials);
        }

        _login(&db_state.pool, db_user).await
    } else {
        Err(LoginError::InvalidCredentials)
    }
}

async fn _login(pool: &Pool, user: User) -> Result<Json<LoginResponse>, LoginError> {
    // TODO: Potentially limit the number of sessions.
    let token = create_session(pool, user.id.unwrap()).await;

    // Not being able to create a session token is not a user error, something's wrong!
    if let Err(_) = token {
        return Err(LoginError::ServerError);
    }

    Ok(Json(LoginResponse {
        token: token.unwrap(),
    }))
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
