use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::core::{
    Account, AccountId, Transaction, TransactionId,
    db::repos::{
        DatabaseError, Pool, is_foreign_key_violation, transactions_repo::TransactionType,
    },
    validators::{validate_greater_than, validate_is_valid_date, validate_not_empty},
};

#[derive(Error, Debug)]
pub enum CreateTransactionError {
    #[error("Invalid price per share, must be greater than 0")]
    InvalidPricePerShare,
    #[error("Invalid share quantity, must be greater than 0")]
    InvalidShareQuantity,
    #[error("Invalid currency, must be a valid ISO 4217 code")]
    InvalidCurrency,
    #[error("Invalid ticker symbol")]
    InvalidTickerSymbol,
    #[error("Invalid date, must be in the format YYYY-MM-DD")]
    InvalidDate,
    #[error("The specified account ID does not exist")]
    AccountNotFound,
    #[error("Database error")]
    DatabaseError(#[from] DatabaseError),
}

/// Attempts to create a new transaction with the given data for the specified user ID.
pub async fn create_transaction(
    pool: &Pool,
    belonging_to_user_id: TransactionId,
    data: &TransactionData,
) -> Result<Transaction, CreateTransactionError> {
    validate_greater_than(data.price_per_share, 0.0)
        .map_err(|_| CreateTransactionError::InvalidPricePerShare)?;
    validate_greater_than(data.share_quantity, 0.0)
        .map_err(|_| CreateTransactionError::InvalidShareQuantity)?;
    // TODO: Validate that the currency is a valid ISO 4217 code.
    validate_not_empty(&data.currency).map_err(|_| CreateTransactionError::InvalidCurrency)?;
    validate_not_empty(&data.ticker_symbol)
        .map_err(|_| CreateTransactionError::InvalidTickerSymbol)?;
    let date =
        validate_is_valid_date(&data.date).map_err(|_| CreateTransactionError::InvalidDate)?;

    let account = Account::by_id(pool, data.account_id).await?;
    if let None = account {
        return Err(CreateTransactionError::AccountNotFound);
    }

    Transaction::insert(
        pool,
        belonging_to_user_id,
        data.account_id,
        data.transaction_type,
        date,
        data.ticker_symbol.as_str(),
        data.share_quantity,
        data.price_per_share,
        data.currency.as_str(),
        data.fees,
    )
    .await
    .map_err(|err| {
        println!("Error creating transaction: {:?}", err);
        if is_foreign_key_violation(&err) {
            CreateTransactionError::AccountNotFound
        } else {
            CreateTransactionError::DatabaseError(err)
        }
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionData {
    pub account_id: AccountId,
    pub transaction_type: TransactionType,
    pub price_per_share: f64,
    pub share_quantity: f64,
    pub fees: f64,
    pub currency: String,
    pub ticker_symbol: String,
    pub date: String,
}
