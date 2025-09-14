use serde::{Deserialize, Serialize};
use time::{Date, PrimitiveDateTime};

use crate::db::repos::{Pool, accounts_repo::AccountId, users_repo::UserId};

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
                RETURNING id
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
        })
    }
}
