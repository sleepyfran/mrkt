use maud::{Markup, html};
use rocket::{State, http::Status, request::FlashMessage};

use crate::{
    core::{
        AccountId,
        accounts::{ListAllAccountsError, list_all_accounts},
        state::CoreState,
    },
    impl_responder,
    site::{
        auth_guard::CookieAuthenticatedUser,
        shared::{NavSection, Shell, empty_state, format_relative_date},
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
    flash: Option<FlashMessage<'_>>,
) -> Result<Markup, ListAllAccountsError> {
    let accounts = list_all_accounts(&db_state.pool, auth_user.user_id).await?;

    Ok(html! {
        (
            Shell::create(
                NavSection::Accounts,
                "Your Accounts",
                html! {
                    div {
                        page-header {
                            page-header-title { "Your Accounts" }
                            page-header-subtitle { "Manage your investment accounts" }
                        }

                        @if accounts.is_empty() {
                            (empty_state(
                                "🏦",
                                "No accounts yet",
                                "Create your first investment account to get started",
                                "Create Your First Account",
                                "/accounts/create"
                            ))
                        } @else {
                            section-controls {
                                h2 { "Account List" }
                                a href="/accounts/create" {
                                    "Add Account"
                                }
                            }

                            table-container {
                                table data-table="content" {
                                    thead {
                                        tr {
                                            th { "Account Name" }
                                            th { "Account ID" }
                                            th { "Created" }
                                            th { "Actions" }
                                        }
                                    }
                                    tbody {
                                        @for account in &accounts {
                                            tr {
                                                td {
                                                    div {
                                                        div { (account.name) }
                                                        div { "Investment Account" }
                                                    }
                                                }
                                                td {
                                                    @if let Some(id) = account.id {
                                                        (format!("#{}", id))
                                                    } @else {
                                                        "N/A"
                                                    }
                                                }
                                                td {
                                                    (format_relative_date(account.created_at))
                                                }
                                                td class="actions" {
                                                    @if let Some(id) = account.id {
                                                        a href=(uri!(transactions::list::list_by_account(id))) class="table-action" {
                                                            "View Transactions"
                                                        }
                                                        " "
                                                        form method="post" action=(format!("/accounts/{}/delete", id)) style="display: inline;"
                                                             onsubmit="return confirm('Are you sure you want to delete this account? This action cannot be undone.');" {
                                                            form-submit data-size="small" data-variant="danger" {
                                                                input
                                                                    type="submit"
                                                                    value="Delete";
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            )
            .attach_flash(flash)
        )
    })
}
