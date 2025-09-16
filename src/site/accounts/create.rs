use maud::{Markup, html};
use rocket::{
    State,
    form::{Form, FromForm},
    response::Redirect,
};

use crate::{
    core::{
        accounts::{CreateAccountError, create_account},
        state::CoreState,
    },
    site::{auth_guard::CookieAuthenticatedUser, shared::base_template},
};

#[derive(FromForm)]
pub struct AccountForm {
    name: String,
}

/// Renders the account creation page.
#[get("/accounts/create")]
pub async fn create_page() -> Markup {
    html! {
        (base_template())
        body class="bg-gray-50 min-h-screen flex items-center justify-center py-12 px-4 sm:px-6 lg:px-8" {
            div class="max-w-md w-full space-y-8" {
                div class="text-center" {
                    h1 class="text-3xl font-bold text-gray-900 mb-2" { "Create New Account" }
                    p class="text-sm text-gray-600" { "Add a new investment account" }
                }

                form method="post" action="/accounts/create" class="mt-8 space-y-6 bg-white p-8 rounded-lg shadow-md" {
                    div class="space-y-4" {
                        div {
                            label for="name" class="block text-sm font-medium text-gray-700 mb-1" { "Account Name" }
                            input
                                type="text"
                                id="name"
                                name="name"
                                required
                                class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition duration-200"
                                placeholder="e.g., My Investment Account, Retirement Fund";
                        }
                    }

                    div class="pt-4" {
                        input
                            type="submit"
                            value="Create Account"
                            class="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 cursor-pointer transition duration-200";
                    }
                }

                div class="text-center mt-4" {
                    a href="/accounts" class="font-medium text-blue-600 hover:text-blue-500" {
                        "← Back to accounts"
                    }
                }
            }
        }
    }
}

/// Handler for account creation form submission.
#[post("/accounts/create", data = "<form>")]
pub async fn create_submit(
    form: Form<AccountForm>,
    db_state: &State<CoreState>,
    auth_user: CookieAuthenticatedUser<'_>,
) -> Result<Redirect, CreateAccountError> {
    create_account(&db_state.pool, auth_user.user_id, &form.name).await?;
    Ok(Redirect::to("/accounts"))
}
