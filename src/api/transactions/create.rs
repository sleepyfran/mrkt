use rocket::{http::Status, serde::json::Json};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{api::auth_guard::SessionToken, impl_responder};

#[derive(Error, Debug)]
enum CreateError {
    #[error("Invalid input")]
    InvalidInput,
    #[error("Database error")]
    DatabaseError,
}

impl_responder! {
    CreateError {
        InvalidInput => Status::BadRequest,
        DatabaseError => Status::InternalServerError,
    }
}

#[post("/", data = "<transaction>")]
pub async fn create(
    _session_token: SessionToken<'_>,
    transaction: Json<TransactionData>,
) -> Result<(), CreateError> {
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionData {
    pub price_per_share: f32,
    pub share_quantity: u32,
    pub fees: f32,
    pub currency: String,
    pub ticker_symbol: String,
    pub date: String,
}
