use rocket::{State, http::Status, delete};

use crate::{
    core::{
        TransactionId,
        state::CoreState,
        transactions::{DeleteTransactionError, delete_transaction},
    },
    api::auth_guard::HeaderAuthenticatedUser,
    impl_responder,
};

impl_responder! {
    DeleteTransactionError {
        DeleteTransactionError::TransactionNotFound => Status::NotFound,
        DeleteTransactionError::DatabaseError(_) => Status::InternalServerError,
    }
}

#[delete("/transactions/<transaction_id>")]
pub async fn delete(
    transaction_id: TransactionId,
    db_state: &State<CoreState>,
    auth_user: HeaderAuthenticatedUser<'_>,
) -> Result<Status, DeleteTransactionError> {
    delete_transaction(&db_state.pool, auth_user.user_id, transaction_id).await?;
    Ok(Status::NoContent)
}