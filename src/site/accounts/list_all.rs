use maud::{Markup, html};
use rocket::{State, http::Status};

use crate::{
    core::{
        AccountId,
        accounts::{ListAllAccountsError, list_all_accounts},
        state::CoreState,
    },
    impl_responder,
    site::{
        auth_guard::CookieAuthenticatedUser,
        shared::{NavSection, Shell},
        transactions,
    },
};

impl_responder! {
    ListAllAccountsError {
        ListAllAccountsError::DatabaseError(_) => Status::InternalServerError,
    }
}

#[get("/accounts")]
pub async fn list_all(
    db_state: &State<CoreState>,
    auth_user: CookieAuthenticatedUser<'_>,
) -> Result<Markup, ListAllAccountsError> {
    let accounts = list_all_accounts(&db_state.pool, auth_user.user_id).await?;

    Ok(html! {
        (
            Shell::create(
                NavSection::Accounts,
                "Account List",
                html! {
                    div class="" {
                        div class="" {
                            h1 class="" { "Your Accounts" }
                            p class="" { "Manage your investment accounts" }
                        }

                        div class="" {
                            div class="" {
                                h2 class="" { "Account List" }
                                a href="/accounts/create"
                                class="" {
                                    "Add Account"
                                }
                            }

                            @if accounts.is_empty() {
                                div class="" {
                                    div class="" {
                                        // Simple icon representation using text
                                        p class="" { "🏦" }
                                    }
                                    h3 class="" { "No accounts yet" }
                                    p class="" { "Create your first investment account to get started" }
                                    a href="/accounts/create"
                                    class="" {
                                        "Create Your First Account"
                                    }
                                }
                            } @else {
                                // Calculate summary values before iterating
                                @let total_accounts = accounts.len();

                                div class="" {
                                    table class="" {
                                        thead class="" {
                                            tr {
                                                th class="" { "Account Name" }
                                                th class="" { "Account ID" }
                                                th class="" { "Created" }
                                                th class="" { "Actions" }
                                            }
                                        }
                                        tbody class="" {
                                            @for account in &accounts {
                                                tr class="" {
                                                    td class="" {
                                                        div class="" {
                                                            div {
                                                                div class="" { (account.name) }
                                                                div class="" { "Investment Account" }
                                                            }
                                                        }
                                                    }
                                                    td class="" {
                                                        @if let Some(id) = account.id {
                                                            (format!("#{}", id))
                                                        } @else {
                                                            "N/A"
                                                        }
                                                    }
                                                    td class="" {
                                                        // Since we don't have created_at field, we'll show a placeholder
                                                        "Recent"
                                                    }
                                                    td class="" {
                                                        @if let Some(id) = account.id {
                                                            a href=(uri!(transactions::list::list_by_account(id)))
                                                            class="" {
                                                                "View Transactions"
                                                            }
                                                        }
                                                        // Edit and delete options can be added later
                                                        span class="" { "•••" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                // Summary section
                                div class="" {
                                    div class="" {
                                        h4 class="" { "Total Accounts" }
                                        p class="" { (total_accounts) }
                                    }
                                    div class="" {
                                        h4 class="" { "Active Accounts" }
                                        p class="" { (total_accounts) }
                                    }
                                }
                            }
                        }
                    }
                }
            )
        )
    })
}
