use diesel::prelude::*;

use crate::db::schema::users;

#[derive(Queryable, Selectable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
    pub id: Option<i32>,
    pub username: String,
    pub hashed_password: String,
}
