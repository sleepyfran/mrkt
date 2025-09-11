use diesel::{Selectable, prelude::Queryable};

use crate::db::{model::user::UserId, schema::accounts};

pub type AccountId = i32;

#[derive(Queryable, Selectable)]
#[diesel(table_name = accounts)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Account {
    pub id: Option<AccountId>,
    /// Reference to the user who owns this account.
    pub owner_id: UserId,
    pub name: String,
}
