use rocket::{State, http::Status, serde::json::Json, response::status};

use crate::{
    api::{auth_guard::HeaderAuthenticatedUser, responses::{ErrorResponse, TickerSymbolErrorResponse}},
    core::{
        Transaction,
        state::CoreState,
        transactions::{CreateTransactionError, TransactionData, create_transaction},
    },
};

impl<'r> rocket::response::Responder<'r, 'static> for CreateTransactionError {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        match self {
            CreateTransactionError::TickerSymbolNotFound { symbol, suggestions } => {
                let response = TickerSymbolErrorResponse {
                    error: format!("Ticker symbol '{}' not found", symbol),
                    invalid_symbol: symbol,
                    suggestions,
                };
                status::BadRequest(Json(response)).respond_to(req)
            }
            _ => {
                let error_response = ErrorResponse {
                    error: self.to_string(),
                };

                let status = match self {
                    CreateTransactionError::InvalidPricePerShare => Status::BadRequest,
                    CreateTransactionError::InvalidShareQuantity => Status::BadRequest,
                    CreateTransactionError::InvalidCurrency => Status::BadRequest,
                    CreateTransactionError::InvalidTickerSymbol => Status::BadRequest,
                    CreateTransactionError::InvalidDate => Status::BadRequest,
                    CreateTransactionError::AccountNotFound => Status::NotFound,
                    CreateTransactionError::MarketDataUnavailable => Status::ServiceUnavailable,
                    CreateTransactionError::DatabaseError(_) => Status::InternalServerError,
                    CreateTransactionError::TickerSymbolNotFound { .. } => unreachable!(),
                };

                (status, Json(error_response)).respond_to(req)
            }
        }
    }
}

#[post("/transactions", data = "<transaction>")]
pub async fn create(
    db_state: &State<CoreState>,
    auth_user: HeaderAuthenticatedUser<'_>,
    transaction: Json<TransactionData>,
) -> Result<Json<Transaction>, CreateTransactionError> {
    create_transaction(
        &db_state.pool, 
        db_state.market_provider.as_ref(),
        auth_user.user_id, 
        &transaction
    )
    .await
    .map(Json)
}
