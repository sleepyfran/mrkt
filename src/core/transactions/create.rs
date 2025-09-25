use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::core::{
    Account, AccountId, Transaction, TransactionId,
    data_sources::market_provider::SymbolSearchResult,
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
    #[error("Ticker symbol '{symbol}' not found")]
    TickerSymbolNotFound {
        symbol: String,
        suggestions: Vec<SymbolSearchResult>,
    },
    #[error("Invalid date, must be in the format YYYY-MM-DD")]
    InvalidDate,
    #[error("The specified account ID does not exist")]
    AccountNotFound,
    #[error("Market data service unavailable")]
    MarketDataUnavailable,
    #[error("Database error")]
    DatabaseError(#[from] DatabaseError),
}

/// Attempts to create a new transaction with the given data for the specified user ID.
pub async fn create_transaction(
    pool: &Pool,
    market_provider: &dyn crate::core::data_sources::MarketProvider,
    belonging_to_user_id: TransactionId,
    data: &TransactionData,
) -> Result<Transaction, CreateTransactionError> {
    // For transfers, price per share can be 0 (since they're free stock grants/deliveries)
    // For buy/sell transactions, price must be greater than 0.
    if !matches!(data.transaction_type, TransactionType::Transfer) {
        validate_greater_than(data.price_per_share, 0.0)
            .map_err(|_| CreateTransactionError::InvalidPricePerShare)?;
    }

    validate_greater_than(data.share_quantity, 0.0)
        .map_err(|_| CreateTransactionError::InvalidShareQuantity)?;
    // TODO: Validate that the currency is a valid ISO 4217 code.
    validate_not_empty(&data.currency).map_err(|_| CreateTransactionError::InvalidCurrency)?;
    validate_not_empty(&data.ticker_symbol)
        .map_err(|_| CreateTransactionError::InvalidTickerSymbol)?;
    let date =
        validate_is_valid_date(&data.date).map_err(|_| CreateTransactionError::InvalidDate)?;

    // Validate ticker symbol exists in market data.
    match market_provider.get_stock_prices(&data.ticker_symbol).await {
        Ok(_) => {
            // Ticker symbol is valid, continue.
        }
        Err(_) => {
            // Ticker symbol not found, try to find suggestions.
            match market_provider.search_symbols(&data.ticker_symbol).await {
                Ok(suggestions) => {
                    // Take up to 5 best matches based on score.
                    let mut sorted_suggestions = suggestions;
                    sorted_suggestions.sort_by(|a, b| {
                        b.match_score
                            .partial_cmp(&a.match_score)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                    sorted_suggestions.truncate(5);

                    return Err(CreateTransactionError::TickerSymbolNotFound {
                        symbol: data.ticker_symbol.clone(),
                        suggestions: sorted_suggestions,
                    });
                }
                Err(_) => {
                    return Err(CreateTransactionError::MarketDataUnavailable);
                }
            }
        }
    }

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
