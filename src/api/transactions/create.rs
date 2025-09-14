use rocket::{State, http::Status, serde::json::Json};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    api::{
        auth_guard::AuthenticatedUser,
        validators::{validate_greater_than, validate_is_valid_date, validate_not_empty},
    },
    db::{
        repos::{
            DatabaseError,
            accounts_repo::{Account, AccountId},
            is_foreign_key_violation,
            transactions_repo::{Transaction, TransactionType},
        },
        state::DbState,
    },
    impl_responder,
};

#[derive(Error, Debug)]
pub enum CreateError {
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

impl_responder! {
    CreateError {
        CreateError::InvalidPricePerShare => Status::BadRequest,
        CreateError::InvalidShareQuantity => Status::BadRequest,
        CreateError::InvalidCurrency => Status::BadRequest,
        CreateError::InvalidTickerSymbol => Status::BadRequest,
        CreateError::InvalidDate => Status::BadRequest,
        CreateError::AccountNotFound => Status::NotFound,
        CreateError::DatabaseError(_) => Status::InternalServerError,
    }
}

#[post("/", data = "<transaction>")]
pub async fn create(
    db_state: &State<DbState>,
    auth_user: AuthenticatedUser<'_>,
    transaction: Json<TransactionData>,
) -> Result<Json<Transaction>, CreateError> {
    validate_greater_than(transaction.price_per_share, 0.0)
        .map_err(|_| CreateError::InvalidPricePerShare)?;
    validate_greater_than(transaction.share_quantity, 0.0)
        .map_err(|_| CreateError::InvalidShareQuantity)?;
    // TODO: Validate that the currency is a valid ISO 4217 code.
    validate_not_empty(&transaction.currency).map_err(|_| CreateError::InvalidCurrency)?;
    validate_not_empty(&transaction.ticker_symbol).map_err(|_| CreateError::InvalidTickerSymbol)?;
    validate_is_valid_date(&transaction.date).map_err(|_| CreateError::InvalidDate)?;

    let account = Account::by_id(&db_state.pool, transaction.account_id).await?;
    if let None = account {
        return Err(CreateError::AccountNotFound);
    }

    let transaction_result = Transaction::insert(
        &db_state.pool,
        auth_user.user_id,
        transaction.account_id,
        transaction.transaction_type,
        transaction.ticker_symbol.as_str(),
        transaction.share_quantity,
        transaction.price_per_share,
        transaction.currency.as_str(),
        transaction.fees,
    )
    .await;

    match transaction_result {
        Ok(transaction) => Ok(Json(transaction)),
        Err(err) => {
            if is_foreign_key_violation(&err) {
                Err(CreateError::AccountNotFound)
            } else {
                Err(CreateError::DatabaseError(err))
            }
        }
    }
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
