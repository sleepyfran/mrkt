use diesel::prelude::Insertable;

use crate::db::schema::users;

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub username: String,
    pub hashed_password: String,
}
