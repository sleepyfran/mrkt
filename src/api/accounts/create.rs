use rocket::{State, http::Status, serde::json::Json};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    api::{auth_guard::AuthenticatedUser, validators::validate_length},
    db::{
        repos::{DatabaseError, accounts_repo::Account},
        state::DbState,
    },
    impl_responder,
};

#[derive(Error, Debug)]
pub enum CreateAccountError {
    #[error("Invalid name")]
    InvalidName,
    #[error("Database error")]
    DatabaseError(#[from] DatabaseError),
}

impl_responder! {
    CreateAccountError {
        CreateAccountError::InvalidName => Status::BadRequest,
        CreateAccountError::DatabaseError(_) => Status::InternalServerError
    }
}

#[post("/", data = "<account>")]
pub async fn create(
    db_state: &State<DbState>,
    auth_user: AuthenticatedUser<'_>,
    account: Json<AccountData>,
) -> Result<Json<Account>, CreateAccountError> {
    validate_length(&account.name, 1, 250).map_err(|_| CreateAccountError::InvalidName)?;

    let account = Account::insert(&db_state.pool, &account.name, auth_user.user_id).await?;
    Ok(Json(account))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountData {
    pub name: String,
}
