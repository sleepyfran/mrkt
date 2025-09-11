use diesel::{Connection, ExpressionMethods, RunQueryDsl, SqliteConnection};

use crate::{
    core::auth::generate_session_token,
    db::{
        model::{
            session::{Session, Token},
            user::UserId,
        },
        schema::sessions,
    },
};

/// Creates a new session for the given user ID, generating a unique session token, saving it
/// to the database, and returning the session token that was created.
pub async fn create_session(
    connection: &mut SqliteConnection,
    user_id: UserId,
) -> Result<Token, diesel::result::Error> {
    let session_token = generate_session_token();

    let new_session = Session {
        id: None,
        user_id,
        token: session_token.clone(),
    };

    connection
        .transaction(|conn| {
            // Delete expired sessions.
            diesel::delete(sessions::table)
                .filter(sessions::expires_at.lt(diesel::dsl::now))
                .execute(conn)?;

            diesel::insert_into(sessions::table)
                .values(new_session)
                .execute(conn)
        })
        .map(|_| session_token)
}
