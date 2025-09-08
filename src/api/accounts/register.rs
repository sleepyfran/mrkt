use diesel::{Connection, RunQueryDsl};
use rocket::serde::json::Json;
use serde::Deserialize;

use crate::api::responses::conflict;
use crate::api::{responses::ApiResult, validators::validate_length};
use crate::db::create_connection;
use crate::db::insertables::NewUser;
use crate::db::schema::users;

#[post("/register", data = "<user>")]
pub async fn register(user: Json<NewUserData>) -> ApiResult<String> {
    validate_length(&user.username, 1, 250)?;
    validate_length(&user.password, 8, 100)?;

    let new_user = NewUser {
        username: user.username.clone(),
        hashed_password: user.password.clone(),
    };

    let mut connection = create_connection();
    let insertion_result = connection.transaction(|conn| {
        diesel::insert_into(users::table)
            .values(new_user)
            .execute(conn)
    });

    match insertion_result {
        Ok(_) => Ok(format!(
            "Hello, {}! Your password is {}",
            user.username, user.password
        )),
        Err(_) => Err(conflict()),
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
