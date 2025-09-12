use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::db::{model::user::UserId, schema::sessions};

pub type SessionId = i32;
pub type Token = String;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = sessions)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Session {
    pub id: Option<SessionId>,
    /// Reference to the user who owns this session.
    pub user_id: UserId,
    /// Unique token for the session.
    pub token: Token,
    /// Timestamp when the session expires. Defaults to 6 months after the creation time, so should
    /// be always available even if typed as optional.
    pub expires_at: Option<NaiveDateTime>,
}
