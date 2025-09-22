use serde::{Deserialize, Serialize};
use time::Date;

use crate::{
    core::db::repos::{Pool, accounts_repo::AccountId, users_repo::UserId},
    core::validators::validate_is_valid_date,
};

pub type TransactionId = i64;

/// Whether the transaction is a buy or sell.
#[derive(Clone, Copy, Deserialize, Serialize, Debug)]
pub enum TransactionType {
    Buy = 0,
    Sell = 1,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Transaction {
    pub id: Option<TransactionId>,
    /// Reference to the user who owns the account.
    pub owner_id: UserId,
    /// Reference to the account that the transaction is associated with.
    pub account_id: AccountId,
    /// Whether the transaction is a buy or sell.
    pub transaction_type: TransactionType,
    /// Date and time of the transaction.
    pub transaction_date: Date,
    /// Ticker symbol of the stock being bought or sold.
    pub ticker_symbol: String,
    /// Number of shares bought or sold.
    pub share_quantity: f64,
    /// Price per share of the stock being bought or sold.
    pub price_per_share: f64,
    /// Currency of the transaction.
    pub currency: String,
    /// Fees associated with the transaction.
    pub fees: f64,
    /// Date of creation of the transaction.
    pub created_at: Date,
}

impl Transaction {
    /// Inserts a new transaction into the database.
    pub async fn insert(
        pool: &Pool,
        owner_id: UserId,
        account_id: AccountId,
        transaction_type: TransactionType,
        transaction_date: Date,
        ticker_symbol: &str,
        share_quantity: f64,
        price_per_share: f64,
        currency: &str,
        fees: f64,
    ) -> Result<Self, sqlx::Error> {
        let numeric_type = transaction_type as i32;
        let row = sqlx::query!(
            r#"
                INSERT INTO transactions (
                    owner_id,
                    account_id,
                    transaction_type,
                    transaction_date,
                    ticker_symbol,
                    quantity,
                    price_per_share,
                    currency,
                    fees
                ) VALUES (
                    $1,
                    $2,
                    $3,
                    $4,
                    $5,
                    $6,
                    $7,
                    $8,
                    $9
                )
                RETURNING id, created_at
            "#,
            owner_id,
            account_id,
            numeric_type,
            transaction_date,
            ticker_symbol,
            share_quantity,
            price_per_share,
            currency,
            fees
        )
        .fetch_one(pool)
        .await?;

        Ok(Self {
            id: row.id,
            owner_id,
            account_id,
            transaction_type,
            transaction_date,
            ticker_symbol: ticker_symbol.to_string(),
            share_quantity,
            price_per_share,
            currency: currency.to_string(),
            fees,
            created_at: row.created_at.date(),
        })
    }

    /// Retrieves all transactions from the database.
    pub async fn get_all(
        pool: &Pool,
        belonging_to_user_id: UserId,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
                SELECT *
                FROM transactions
                WHERE owner_id = $1
            "#,
            belonging_to_user_id
        )
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let parsed_date = validate_is_valid_date(&row.transaction_date).unwrap();

                Self {
                    id: row.id,
                    owner_id: row.owner_id,
                    account_id: row.account_id,
                    transaction_type: match row.transaction_type {
                        0 => TransactionType::Buy,
                        1 => TransactionType::Sell,
                        _ => panic!("Invalid transaction type in database"),
                    },
                    transaction_date: parsed_date,
                    ticker_symbol: row.ticker_symbol,
                    share_quantity: row.quantity,
                    price_per_share: row.price_per_share,
                    currency: row.currency,
                    fees: row.fees,
                    created_at: row.created_at.date(),
                }
            })
            .collect())
    }

    /// Retrieves all transactions for a specific account.
    pub async fn get_by_account(
        pool: &Pool,
        belonging_to_user_id: UserId,
        account_id: AccountId,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
                SELECT *
                FROM transactions
                WHERE owner_id = $1 AND account_id = $2
                ORDER BY transaction_date DESC
            "#,
            belonging_to_user_id,
            account_id
        )
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let parsed_date = validate_is_valid_date(&row.transaction_date).unwrap();

                Self {
                    id: row.id,
                    owner_id: row.owner_id,
                    account_id: row.account_id,
                    transaction_type: match row.transaction_type {
                        0 => TransactionType::Buy,
                        1 => TransactionType::Sell,
                        _ => panic!("Invalid transaction type in database"),
                    },
                    transaction_date: parsed_date,
                    ticker_symbol: row.ticker_symbol,
                    share_quantity: row.quantity,
                    price_per_share: row.price_per_share,
                    currency: row.currency,
                    fees: row.fees,
                    created_at: row.created_at.date(),
                }
            })
            .collect())
    }

    /// Retrieves a transaction by its ID and ensures it belongs to the specified user.
    pub async fn by_id(
        pool: &Pool,
        transaction_id: TransactionId,
        belonging_to_user_id: UserId,
    ) -> Result<Option<Self>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
                SELECT *
                FROM transactions
                WHERE id = $1 AND owner_id = $2
            "#,
            transaction_id,
            belonging_to_user_id
        )
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            let parsed_date = validate_is_valid_date(&row.transaction_date).unwrap();

            Ok(Some(Self {
                id: Some(row.id),
                owner_id: row.owner_id,
                account_id: row.account_id,
                transaction_type: match row.transaction_type {
                    0 => TransactionType::Buy,
                    1 => TransactionType::Sell,
                    _ => panic!("Invalid transaction type in database"),
                },
                transaction_date: parsed_date,
                ticker_symbol: row.ticker_symbol,
                share_quantity: row.quantity,
                price_per_share: row.price_per_share,
                currency: row.currency,
                fees: row.fees,
                created_at: row.created_at.date(),
            }))
        } else {
            Ok(None)
        }
    }

    /// Deletes a transaction by its ID.
    pub async fn delete(pool: &Pool, transaction_id: TransactionId) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
                DELETE FROM transactions
                WHERE id = $1
            "#,
            transaction_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
