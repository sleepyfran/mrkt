use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use crate::api::responses::ApiResult;

#[post("/create", data = "<transaction>")]
pub async fn create(transaction: Json<TransactionData>) -> ApiResult<()> {
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionData {
    pub id: String,
    pub price_per_share: f32,
    pub share_quantity: u32,
    pub fees: f32,
    pub currency: String,
    pub ticker_symbol: String,
    pub date: String,
}
