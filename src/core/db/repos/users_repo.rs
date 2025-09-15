use sqlx::SqlitePool;

pub type UserId = i64;

/// Represents a user in the database.
pub struct User {
    pub id: Option<UserId>,
    pub username: String,
    pub hashed_password: String,
}

impl User {
    /// Inserts a new user into the database and returns the inserted user's ID.
    pub async fn insert(
        pool: &SqlitePool,
        username: &str,
        hashed_password: &str,
    ) -> Result<UserId, sqlx::Error> {
        let id = sqlx::query!(
            r#"
                INSERT INTO users (username, hashed_password)
                VALUES (?, ?)
            "#,
            username,
            hashed_password
        )
        .execute(pool)
        .await?
        .last_insert_rowid();

        Ok(id)
    }

    /// Attempts to find a user by their username.
    pub async fn by_username(
        pool: &SqlitePool,
        username: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"
                SELECT id, username, hashed_password
                FROM users
                WHERE username = ?
            "#,
            username
        )
        .fetch_optional(pool)
        .await
    }
}
