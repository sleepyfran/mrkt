use rocket::{State, post, response::Redirect};
use rocket::response::Flash;

use crate::{
    core::{
        TransactionId,
        state::CoreState,
        transactions::{DeleteTransactionError, delete_transaction},
    },
    site::auth_guard::CookieAuthenticatedUser,
};

#[post("/transactions/<transaction_id>/delete")]
pub async fn delete(
    transaction_id: TransactionId,
    db_state: &State<CoreState>,
    auth_user: CookieAuthenticatedUser<'_>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match delete_transaction(&db_state.pool, auth_user.user_id, transaction_id).await {
        Ok(()) => {
            Ok(Flash::success(
                Redirect::to("/transactions"),
                "Transaction deleted successfully.",
            ))
        }
        Err(DeleteTransactionError::TransactionNotFound) => {
            Err(Flash::error(
                Redirect::to("/transactions"),
                "Transaction not found or you don't have permission to delete it.",
            ))
        }
        Err(DeleteTransactionError::DatabaseError(_)) => {
            Err(Flash::error(
                Redirect::to("/transactions"),
                "An error occurred while deleting the transaction.",
            ))
        }
    }
}