use rocket::State;
use rocket::http::Status;
use rocket::serde::json::Json;
use serde::Deserialize;

use crate::core::{
    auth::{RegisterError, register as _register},
    state::CoreState,
};

use crate::impl_responder;

impl_responder! {
    RegisterError {
        RegisterError::InvalidUsername => Status::BadRequest,
        RegisterError::InvalidPassword => Status::BadRequest,
        RegisterError::UsernameAlreadyExists => Status::Conflict,
        RegisterError::InternalServerError => Status::InternalServerError,
    }
}

#[post("/users/register", data = "<user>")]
pub async fn register(
    db_state: &State<CoreState>,
    user: Json<NewUserData>,
) -> Result<(), RegisterError> {
    _register(&db_state.pool, &user.username, &user.password).await
}

/// Represents a new user to be registered.
#[derive(Debug, Clone, Default, FromForm, Deserialize)]
pub struct NewUserData {
    /// The username of the new user.
    pub username: String,
    /// The unhashed password of the new user.
    pub password: String,
}
