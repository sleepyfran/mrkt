use diesel::{ExpressionMethods, QueryDsl};
use diesel::{OptionalExtension, RunQueryDsl, SelectableHelper};
use rocket::serde::json::Json;
use serde::Deserialize;

use crate::api::responses::{ApiResult, server_error, unauthorized};
use crate::db::create_connection;
use crate::db::model::User;
use crate::db::schema::users;

#[post("/login", data = "<user>")]
pub async fn login(user: Json<LoginData>) -> ApiResult<String> {
    let mut connection = create_connection();

    let db_user = users::table
        .filter(users::username.eq(&user.username))
        .select(User::as_select())
        .first::<User>(&mut connection)
        .optional()
        .map_err(|_| server_error())?;

    if let Some(db_user) = db_user {
        if user.password != db_user.hashed_password {
            Err(unauthorized())
        } else {
            Ok(format!("User {} logged in", user.username))
        }
    } else {
        Err(unauthorized())
    }
}

/// Represents a user attempting to login.
#[derive(Debug, Clone, Default, FromForm, Deserialize)]
pub struct LoginData {
    /// The username of the user.
    pub username: String,
    /// The unhashed password of the user.
    pub password: String,
}
