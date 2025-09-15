use rocket::{State, http::Status, serde::json::Json};
use serde::{Deserialize, Serialize};

use crate::{
    api::auth_guard::HeaderAuthenticatedUser,
    core::{
        Account,
        accounts::{CreateAccountError, create_account},
        state::CoreState,
    },
    impl_responder,
};

impl_responder! {
    CreateAccountError {
        CreateAccountError::InvalidName => Status::BadRequest,
        CreateAccountError::DatabaseError(_) => Status::InternalServerError
    }
}

#[post("/accounts", data = "<account>")]
pub async fn create(
    db_state: &State<CoreState>,
    auth_user: HeaderAuthenticatedUser<'_>,
    account: Json<AccountData>,
) -> Result<Json<Account>, CreateAccountError> {
    let account = create_account(&db_state.pool, auth_user.user_id, &account.name).await?;
    Ok(Json(account))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountData {
    pub name: String,
}
