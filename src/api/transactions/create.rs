use rocket::{State, http::Status, serde::json::Json};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    api::{
        auth_guard::AuthenticatedUser,
        validators::{validate_greater_than, validate_is_valid_date, validate_not_empty},
    },
    db::{
        repos::accounts_repo::{Account, AccountId},
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
    DatabaseError,
}

impl_responder! {
    CreateError {
        InvalidPricePerShare => Status::BadRequest,
        InvalidShareQuantity => Status::BadRequest,
        InvalidCurrency => Status::BadRequest,
        InvalidTickerSymbol => Status::BadRequest,
        InvalidDate => Status::BadRequest,
        AccountNotFound => Status::NotFound,
        DatabaseError => Status::InternalServerError,
    }
}

#[post("/", data = "<transaction>")]
pub async fn create(
    db_state: &State<DbState>,
    auth_user: AuthenticatedUser<'_>,
    transaction: Json<TransactionData>,
) -> Result<(), CreateError> {
    validate_greater_than(transaction.price_per_share, 0.0)
        .map_err(|_| CreateError::InvalidPricePerShare)?;
    validate_greater_than(transaction.share_quantity, 0.0)
        .map_err(|_| CreateError::InvalidShareQuantity)?;
    // TODO: Validate that the currency is a valid ISO 4217 code.
    validate_not_empty(&transaction.currency).map_err(|_| CreateError::InvalidCurrency)?;
    validate_not_empty(&transaction.ticker_symbol).map_err(|_| CreateError::InvalidTickerSymbol)?;
    validate_is_valid_date(&transaction.date).map_err(|_| CreateError::InvalidDate)?;

    let account = Account::by_id(&db_state.pool, transaction.account_id)
        .await
        .map_err(|_| CreateError::DatabaseError)?;

    if let None = account {
        return Err(CreateError::AccountNotFound);
    }

    // let transaction = Transaction {
    //     id: None,
    //     owner_id: auth_user.user_id,
    //     account_id: transaction.account_id,
    //     transaction_type: transaction.transaction_type,
    //     ticker_symbol: transaction.ticker_symbol.clone(),
    //     transaction_date: transaction.date.clone(),
    //     quantity: transaction.share_quantity,
    //     price_per_share: transaction.price_per_share,
    //     currency: transaction.currency.clone(),
    //     fees: transaction.fees,
    // };

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionData {
    pub account_id: AccountId,
    // pub transaction_type: TransactionType,
    pub price_per_share: f32,
    pub share_quantity: f32,
    pub fees: f32,
    pub currency: String,
    pub ticker_symbol: String,
    pub date: String,
}
