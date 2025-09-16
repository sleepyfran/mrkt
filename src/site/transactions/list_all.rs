use maud::{Markup, html};
use rocket::{State, http::Status};

use crate::{
    core::{
        state::CoreState,
        transactions::{ListAllTransactionsError, list_all_transactions},
    },
    impl_responder,
    site::auth_guard::CookieAuthenticatedUser,
};

impl_responder! {
    ListAllTransactionsError {
        ListAllTransactionsError::DatabaseError(_) => Status::InternalServerError,
    }
}

#[get("/transactions")]
pub async fn list_all(
    db_state: &State<CoreState>,
    auth_user: CookieAuthenticatedUser<'_>,
) -> Result<Markup, ListAllTransactionsError> {
    let transactions = list_all_transactions(&db_state.pool, auth_user.user_id).await?;

    Ok(html! {
      @for transaction in transactions {
        p { (format!("{:?}", transaction)) }
      }
    })
}
