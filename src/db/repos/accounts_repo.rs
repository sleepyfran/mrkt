use crate::db::repos::{DatabaseError, Pool, users_repo::UserId};

pub type AccountId = i64;

/// Represents an account in the database.
pub struct Account {
    pub id: AccountId,
    /// Reference to the user who owns this account.
    pub owner_id: UserId,
    pub name: String,
}

impl Account {
    pub async fn by_id(pool: &Pool, id: AccountId) -> Result<Option<Self>, DatabaseError> {
        let row = sqlx::query!(
            r#"
                SELECT *
                FROM accounts
                WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|row| Account {
            id: row.id,
            owner_id: row.owner_id,
            name: row.name,
        }))
    }
}
