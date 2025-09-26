use rocket::response::Flash;
use rocket::{State, post, response::Redirect};

use crate::site::shared::ShellFlash;
use crate::{
    core::{
        AccountId,
        accounts::{DeleteAccountError, delete_account},
        state::CoreState,
    },
    site::auth_guard::CookieAuthenticatedUser,
};

#[post("/accounts/<account_id>/delete")]
pub async fn delete(
    account_id: AccountId,
    db_state: &State<CoreState>,
    auth_user: CookieAuthenticatedUser<'_>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match delete_account(&db_state.pool, auth_user.user_id, account_id).await {
        Ok(()) => Ok(Flash::success(
            Redirect::to("/accounts"),
            ShellFlash::to_flash_message("Account deleted successfully."),
        )),
        Err(DeleteAccountError::AccountNotFound) => Err(Flash::error(
            Redirect::to("/accounts"),
            ShellFlash::to_flash_message(
                "Account not found or you don't have permission to delete it.",
            ),
        )),
        Err(DeleteAccountError::DatabaseError(_)) => Err(Flash::error(
            Redirect::to("/accounts"),
            ShellFlash::to_flash_message("An error occurred while deleting the account."),
        )),
    }
}
