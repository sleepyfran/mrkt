use diesel::{ExpressionMethods, QueryDsl, SqliteConnection};
use diesel::{OptionalExtension, RunQueryDsl, SelectableHelper};
use rocket::http::Status;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::api::accounts::session::create_session;
use crate::core::auth::verify_password;
use crate::db::create_connection;
use crate::db::model::user::User;
use crate::db::schema::users;
use crate::impl_responder;

#[derive(Error, Debug)]
enum LoginError {
    #[error("Invalid username or password")]
    InvalidCredentials,
    #[error("Internal server error")]
    ServerError,
}

impl_responder! {
    LoginError {
        InvalidCredentials => Status::Unauthorized,
        ServerError => Status::InternalServerError,
    }
}

#[post("/login", data = "<login_data>")]
pub async fn login(login_data: Json<LoginData>) -> Result<Json<LoginResponse>, LoginError> {
    let mut connection = create_connection();

    let db_user = users::table
        .filter(users::username.eq(&login_data.username))
        .select(User::as_select())
        .first::<User>(&mut connection)
        .optional()
        .map_err(|_| LoginError::ServerError)?;

    if let Some(db_user) = db_user {
        let password_is_correct = verify_password(&login_data.password, &db_user.hashed_password);
        if !password_is_correct {
            return Err(LoginError::InvalidCredentials);
        }

        _login(&mut connection, db_user).await
    } else {
        Err(LoginError::InvalidCredentials)
    }
}

async fn _login(
    connection: &mut SqliteConnection,
    user: User,
) -> Result<Json<LoginResponse>, LoginError> {
    // TODO: Potentially limit the number of sessions.
    let token = create_session(connection, user.id.unwrap()).await;

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
