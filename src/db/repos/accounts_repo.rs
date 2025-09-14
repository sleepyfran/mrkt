use serde::{Deserialize, Serialize};

use crate::db::repos::{DatabaseError, Pool, users_repo::UserId};

pub type AccountId = i64;

/// Represents an account in the database.
#[derive(Serialize, Deserialize, Debug)]
pub struct Account {
    pub id: Option<AccountId>,
    /// Reference to the user who owns this account.
    pub owner_id: UserId,
    pub name: String,
}

impl Account {
    /// Inserts a new account into the database with the specified name.
    pub async fn insert(pool: &Pool, name: &str, owner_id: UserId) -> Result<Self, DatabaseError> {
        let row = sqlx::query!(
            r#"
                INSERT INTO accounts (owner_id, name)
                VALUES ($1, $2)
                RETURNING id
            "#,
            owner_id,
            name
        )
        .fetch_one(pool)
        .await?;

        Ok(Account {
            id: row.id,
            owner_id,
            name: name.to_string(),
        })
    }

    /// Returns an account by its ID.
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
            id: Some(row.id),
            owner_id: row.owner_id,
            name: row.name,
        }))
    }
}
