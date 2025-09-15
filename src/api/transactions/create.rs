use rocket::{State, http::Status, serde::json::Json};

use crate::{
    api::auth_guard::HeaderAuthenticatedUser,
    core::{
        Transaction,
        state::CoreState,
        transactions::{CreateTransactionError, TransactionData, create_transaction},
    },
    impl_responder,
};

impl_responder! {
    CreateTransactionError {
        CreateTransactionError::InvalidPricePerShare => Status::BadRequest,
        CreateTransactionError::InvalidShareQuantity => Status::BadRequest,
        CreateTransactionError::InvalidCurrency => Status::BadRequest,
        CreateTransactionError::InvalidTickerSymbol => Status::BadRequest,
        CreateTransactionError::InvalidDate => Status::BadRequest,
        CreateTransactionError::AccountNotFound => Status::NotFound,
        CreateTransactionError::DatabaseError(_) => Status::InternalServerError,
    }
}

#[post("/transactions", data = "<transaction>")]
pub async fn create(
    db_state: &State<CoreState>,
    auth_user: HeaderAuthenticatedUser<'_>,
    transaction: Json<TransactionData>,
) -> Result<Json<Transaction>, CreateTransactionError> {
    create_transaction(&db_state.pool, auth_user.user_id, &transaction)
        .await
        .map(Json)
}
