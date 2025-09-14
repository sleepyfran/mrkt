use time::OffsetDateTime;

use crate::db::repos::{Pool, users_repo::UserId};

pub type SessionId = i64;
pub type Token = String;

pub struct Session {
    pub id: Option<SessionId>,
    /// Reference to the user who owns this session.
    pub user_id: UserId,
    /// Unique token for the session.
    pub token: Token,
    /// Timestamp when the session expires. Defaults to 6 months after the creation time, so should
    /// always be avilable even if typed as optional.
    pub expires_at: Option<OffsetDateTime>,
}

impl Session {
    /// Inserts a new session into the database.
    pub async fn insert(pool: &Pool, user_id: UserId, token: Token) -> Result<Self, sqlx::Error> {
        let row = sqlx::query!(
            r#"
                INSERT INTO sessions (user_id, token)
                VALUES ($1, $2)
                RETURNING id, expires_at
            "#,
            user_id,
            token
        )
        .fetch_one(pool)
        .await?;

        Ok(Self {
            id: row.id,
            user_id,
            token,
            expires_at: row.expires_at,
        })
    }

    /// Cleans up expired sessions from the database.
    pub async fn cleanup_expired(pool: &Pool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
                DELETE FROM sessions
                WHERE expires_at < datetime('now')
            "#
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Retrieves a session from the database by its associated token.
    pub async fn by_token(pool: &Pool, token: &str) -> Result<Option<Self>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
                SELECT * FROM sessions
                WHERE token = $1
            "#,
            token
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|row| Self {
            id: row.id,
            user_id: row.user_id,
            token: row.token,
            expires_at: row.expires_at,
        }))
    }
}
