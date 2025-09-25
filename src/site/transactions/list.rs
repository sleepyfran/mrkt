use maud::{Markup, html};
use rocket::{State, http::Status, request::FlashMessage};

use crate::{
    core::{
        Account, AccountId, Transaction, TransactionType,
        state::CoreState,
        transactions::{
            ListAllTransactionsError, ListByAccountTransactionsError, list_all_transactions,
            list_transactions_by_account,
        },
    },
    impl_responder,
    site::{
        auth_guard::CookieAuthenticatedUser,
        shared::{NavSection, Shell, empty_state},
    },
};

impl_responder! {
    ListAllTransactionsError {
        ListAllTransactionsError::DatabaseError(_) => Status::InternalServerError,
    }
}

impl_responder! {
    ListByAccountTransactionsError {
        ListByAccountTransactionsError::DatabaseError(_) => Status::InternalServerError,
    }
}

/// Renders the transactions template with the provided transactions and accounts
fn render_transactions(
    transactions: &[Transaction],
    accounts: &[Account],
    page_title: &str,
    page_subtitle: &str,
    flash: Option<FlashMessage<'_>>,
) -> Markup {
    html! {
        (
            Shell::create(NavSection::Transactions, page_title, html! {
                div {
                    page-header {
                        page-header-title { (page_title) }
                        page-header-subtitle { (page_subtitle) }
                    }

                    @if transactions.is_empty() {
                        (empty_state(
                            "📊",
                            "No transactions yet",
                            "Get started by adding your first stock transaction",
                            "Add Your First Transaction",
                            "/transactions/create"
                        ))
                    } @else {
                        // Calculate summary values before iterating
                        @let total_transactions = transactions.len();
                        @let buy_count = transactions.iter().filter(|t| matches!(t.transaction_type, TransactionType::Buy)).count();
                        @let sell_count = transactions.iter().filter(|t| matches!(t.transaction_type, TransactionType::Sell)).count();
                        @let transfer_count = transactions.iter().filter(|t| matches!(t.transaction_type, TransactionType::Transfer)).count();

                        section-controls {
                            h2 { "Transaction History" }
                            a href="/transactions/create" {
                                "Add Transaction"
                            }
                        }

                        summary-section {
                            div {
                                h4 { "Total Transactions" }
                                p { (total_transactions) }
                            }
                            div {
                                h4 { "Buy Orders" }
                                p { (buy_count) }
                            }
                            div {
                                h4 { "Sell Orders" }
                                p { (sell_count) }
                            }
                            div {
                                h4 { "Transfers" }
                                p { (transfer_count) }
                            }
                        }

                        table-container {
                            table data-table="content" {
                                thead {
                                    tr {
                                        th { "Date" }
                                        th { "Type" }
                                        th { "Symbol" }
                                        th { "Shares" }
                                        th { "Price" }
                                        th { "Total" }
                                        th { "Account" }
                                        th { "Fees" }
                                        th { "Actions" }
                                    }
                                }
                                tbody {
                                    @for transaction in transactions {
                                        tr {
                                            td {
                                                (transaction.transaction_date)
                                            }
                                            td {
                                                @match transaction.transaction_type {
                                                    TransactionType::Buy => {
                                                        status-badge data-variant="primary" {
                                                            "Buy"
                                                        }
                                                    }
                                                    TransactionType::Sell => {
                                                        status-badge data-variant="danger" {
                                                            "Sell"
                                                        }
                                                    }
                                                    TransactionType::Transfer => {
                                                        status-badge data-variant="secondary" {
                                                            "Transfer"
                                                        }
                                                    }
                                                }
                                            }
                                            td {
                                                (transaction.ticker_symbol)
                                            }
                                            td {
                                                (format!("{:.4}", transaction.share_quantity))
                                            }
                                            td {
                                                (format!("{:.2} {}", transaction.price_per_share, transaction.currency))
                                            }
                                            td {
                                                (format!("{:.2} {}",
                                                    transaction.share_quantity * transaction.price_per_share + transaction.fees,
                                                    transaction.currency))
                                            }
                                            td {
                                                @let account_name = accounts.iter()
                                                    .find(|acc| acc.id == Some(transaction.account_id))
                                                    .map(|acc| acc.name.as_str())
                                                    .unwrap_or("Unknown Account");
                                                (account_name)
                                            }
                                            td {
                                                @if transaction.fees > 0.0 {
                                                    (format!("{:.2} {}", transaction.fees, transaction.currency))
                                                } @else {
                                                    "-"
                                                }
                                            }
                                            td {
                                                @if let Some(transaction_id) = transaction.id {
                                                    form method="post" action=(format!("/transactions/{}/delete", transaction_id)) style="display: inline;"
                                                         onsubmit="return confirm('Are you sure you want to delete this transaction? This action cannot be undone.');" {
                                                        form-submit data-size="small" data-variant="danger" {
                                                            input
                                                                type="submit"
                                                                value="Delete";
                                                        }
                                                    }
                                                } @else {
                                                    "-"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            })
            .attach_flash(flash)
            .add_stylesheet("transactions.css")
        )
    }
}

#[get("/transactions")]
pub async fn list_all(
    db_state: &State<CoreState>,
    auth_user: CookieAuthenticatedUser<'_>,
    flash: Option<FlashMessage<'_>>,
) -> Result<Markup, ListAllTransactionsError> {
    let transactions = list_all_transactions(&db_state.pool, auth_user.user_id).await?;

    let accounts = Account::for_user(&db_state.pool, auth_user.user_id)
        .await
        .map_err(|_| ListAllTransactionsError::DatabaseError(sqlx::Error::RowNotFound))?;

    Ok(render_transactions(
        &transactions,
        &accounts,
        "Your Transactions",
        "View and manage your stock transactions",
        flash,
    ))
}

#[get("/transactions/<account_id>")]
pub async fn list_by_account(
    account_id: AccountId,
    db_state: &State<CoreState>,
    auth_user: CookieAuthenticatedUser<'_>,
    flash: Option<FlashMessage<'_>>,
) -> Result<Markup, ListByAccountTransactionsError> {
    // Verify that the account belongs to the user
    let account = Account::by_id(&db_state.pool, account_id)
        .await
        .map_err(|_| ListByAccountTransactionsError::DatabaseError(sqlx::Error::RowNotFound))?;

    if let Some(account) = &account {
        if account.owner_id != auth_user.user_id {
            return Err(ListByAccountTransactionsError::DatabaseError(
                sqlx::Error::RowNotFound,
            ));
        }
    } else {
        return Err(ListByAccountTransactionsError::DatabaseError(
            sqlx::Error::RowNotFound,
        ));
    }

    let transactions =
        list_transactions_by_account(&db_state.pool, auth_user.user_id, account_id).await?;

    let accounts = Account::for_user(&db_state.pool, auth_user.user_id)
        .await
        .map_err(|_| ListByAccountTransactionsError::DatabaseError(sqlx::Error::RowNotFound))?;

    let account_name = account.as_ref().unwrap().name.clone();

    Ok(render_transactions(
        &transactions,
        &accounts,
        &format!("Transactions for {}", account_name),
        &format!("View transactions for the {} account", account_name),
        flash,
    ))
}
