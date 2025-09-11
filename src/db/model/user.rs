use diesel::prelude::*;

use crate::db::schema::users;

pub type UserId = i32;

#[derive(Queryable, Selectable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
    pub id: Option<UserId>,
    pub username: String,
    pub hashed_password: String,
}
