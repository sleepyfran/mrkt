use rocket::State;
use rocket::http::Status;
use rocket::serde::json::Json;
use serde::Deserialize;
use thiserror::Error;

use crate::api::validators::validate_length;
use crate::core::auth::hash_password;
use crate::db::repos::is_unique_constraint_violation;
use crate::db::repos::users_repo::User;
use crate::db::state::DbState;
use crate::impl_responder;

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

impl_responder! {
    RegisterError {
        RegisterError::InvalidUsername => Status::BadRequest,
        RegisterError::InvalidPassword => Status::BadRequest,
        RegisterError::UsernameAlreadyExists => Status::Conflict,
        RegisterError::InternalServerError => Status::InternalServerError,
    }
}

#[post("/register", data = "<user>")]
pub async fn register(
    db_state: &State<DbState>,
    user: Json<NewUserData>,
) -> Result<(), RegisterError> {
    validate_length(&user.username, 1, 250).map_err(|_| RegisterError::InvalidUsername)?;
    validate_length(&user.password, 8, 100).map_err(|_| RegisterError::InvalidPassword)?;

    let hashed_password = hash_password(&user.password);
    let insertion_result = User::insert(&db_state.pool, &user.username, &hashed_password).await;

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

/// Represents a new user to be registered.
#[derive(Debug, Clone, Default, FromForm, Deserialize)]
pub struct NewUserData {
    /// The username of the new user.
    pub username: String,
    /// The unhashed password of the new user.
    pub password: String,
}
