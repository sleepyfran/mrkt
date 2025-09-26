use serde::{Deserialize, Serialize};
use time::Date;

use crate::core::db::repos::{DatabaseError, Pool, users_repo::UserId};

pub type AccountId = i64;

/// Represents an account in the database.
#[derive(Serialize, Deserialize, Debug)]
pub struct Account {
    pub id: Option<AccountId>,
    /// Reference to the user who owns this account.
    pub owner_id: UserId,
    pub name: String,
    pub created_at: Date,
}

impl Account {
    /// Inserts a new account into the database with the specified name.
    pub async fn insert(pool: &Pool, name: &str, owner_id: UserId) -> Result<Self, DatabaseError> {
        let row = sqlx::query!(
            r#"
                INSERT INTO accounts (owner_id, name)
                VALUES ($1, $2)
                RETURNING id, created_at
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
            created_at: row.created_at.date(),
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
            created_at: row.created_at.date(),
        }))
    }

    /// Returns all accounts belonging to a specific user.
    pub async fn for_user(pool: &Pool, user_id: UserId) -> Result<Vec<Self>, DatabaseError> {
        let rows = sqlx::query!(
            r#"
                SELECT *
                FROM accounts
                WHERE owner_id = $1
                ORDER BY name
            "#,
            user_id
        )
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| Account {
                id: Some(row.id),
                owner_id: row.owner_id,
                name: row.name,
                created_at: row.created_at.date(),
            })
            .collect())
    }

    /// Deletes an account by its ID.
    pub async fn delete(pool: &Pool, id: AccountId) -> Result<(), DatabaseError> {
        sqlx::query!(
            r#"
                DELETE FROM accounts
                WHERE id = $1
            "#,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
