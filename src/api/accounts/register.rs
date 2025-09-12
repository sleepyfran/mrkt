use diesel::{Connection, RunQueryDsl};
use rocket::http::Status;
use rocket::serde::json::Json;
use serde::Deserialize;
use thiserror::Error;

use crate::api::validators::validate_length;
use crate::core::auth::hash_password;
use crate::db::create_connection;
use crate::db::insertables::NewUser;
use crate::db::schema::users;
use crate::impl_responder;

#[derive(Error, Debug)]
pub enum RegisterError {
    #[error("Invalid username")]
    InvalidUsername,
    #[error("Invalid password")]
    InvalidPassword,
    #[error("Username already exists")]
    UsernameAlreadyExists,
}

impl_responder! {
    RegisterError {
        InvalidUsername => Status::BadRequest,
        InvalidPassword => Status::BadRequest,
        UsernameAlreadyExists => Status::Conflict,
    }
}

#[post("/register", data = "<user>")]
pub async fn register(user: Json<NewUserData>) -> Result<(), RegisterError> {
    validate_length(&user.username, 1, 250).map_err(|_| RegisterError::InvalidUsername)?;
    validate_length(&user.password, 8, 100).map_err(|_| RegisterError::InvalidPassword)?;

    let hashed_password = hash_password(&user.password);
    let new_user = NewUser {
        username: user.username.clone(),
        hashed_password,
    };

    let mut connection = create_connection();
    let insertion_result = connection.transaction(|conn| {
        diesel::insert_into(users::table)
            .values(new_user)
            .execute(conn)
    });

    match insertion_result {
        Ok(_) => Ok(()),
        Err(_) => Err(RegisterError::UsernameAlreadyExists),
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
