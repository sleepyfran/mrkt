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
    site::{
        auth_guard::CookieAuthenticatedUser,
        shared::{NavSection, Shell},
    },
};

#[derive(FromForm)]
pub struct AccountForm {
    name: String,
}

/// Renders the account creation page.
#[get("/accounts/create")]
pub async fn create_page() -> Markup {
    html! {
        (Shell::create(
            NavSection::UserManagement,
            "Register",
            html! {
                div class="" {
                    div class="" {
                        h1 class="" { "Create New Account" }
                        p class="" { "Add a new investment account" }
                    }

                    form method="post" action="/accounts/create" class="" {
                        div class="" {
                            div {
                                label for="name" class="" { "Account Name" }
                                input
                                    type="text"
                                    id="name"
                                    name="name"
                                    required
                                    class=""
                                    placeholder="e.g., My Investment Account, Retirement Fund";
                            }
                        }

                        div class="" {
                            input
                                type="submit"
                                value="Create Account"
                                class="";
                        }
                    }

                    div class="" {
                        a href="/accounts" class="" {
                            "← Back to accounts"
                        }
                    }
                }
            }
        ))
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
