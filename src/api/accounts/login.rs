use diesel::{ExpressionMethods, QueryDsl, SqliteConnection};
use diesel::{OptionalExtension, RunQueryDsl, SelectableHelper};
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use crate::api::accounts::session::create_session;
use crate::api::responses::{ApiResult, server_error, unauthorized};
use crate::core::auth::verify_password;
use crate::db::create_connection;
use crate::db::model::user::User;
use crate::db::schema::users;

#[post("/login", data = "<login_data>")]
pub async fn login(login_data: Json<LoginData>) -> ApiResult<Json<LoginResponse>> {
    let mut connection = create_connection();

    let db_user = users::table
        .filter(users::username.eq(&login_data.username))
        .select(User::as_select())
        .first::<User>(&mut connection)
        .optional()
        .map_err(|_| server_error())?;

    if let Some(db_user) = db_user {
        let password_is_correct = verify_password(&login_data.password, &db_user.hashed_password);
        if !password_is_correct {
            return Err(unauthorized());
        }

        _login(&mut connection, db_user).await
    } else {
        Err(unauthorized())
    }
}

async fn _login(connection: &mut SqliteConnection, user: User) -> ApiResult<Json<LoginResponse>> {
    // TODO: Potentially limit the number of sessions.
    let token = create_session(connection, user.id.unwrap()).await;

    // Not being able to create a session token is not a user error, something's wrong!
    if let Err(_) = token {
        return Err(server_error());
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
