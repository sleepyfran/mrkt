use maud::{Markup, html};
use rocket::{State, http::Status};

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
        shared::{NavSection, Shell},
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
) -> Markup {
    html! {
        (
            Shell::create(NavSection::Transactions, page_title, html! {
                div class="" {
                    div class="" {
                        h1 class="" { (page_title) }
                        p class="" { (page_subtitle) }
                    }

                    div class="" {
                        div class="" {
                            h2 class="" { "Transaction History" }
                            a href="/transactions/create"
                              class="" {
                                "Add Transaction"
                            }
                        }

                        @if transactions.is_empty() {
                            div class="" {
                                div class="" {
                                    // Simple icon representation using text
                                    p class="" { "📊" }
                                }
                                h3 class="" { "No transactions yet" }
                                p class="" { "Get started by adding your first stock transaction" }
                                a href="/transactions/create"
                                  class="" {
                                    "Add Your First Transaction"
                                }
                            }
                        } @else {
                            // Calculate summary values before iterating
                            @let total_transactions = transactions.len();
                            @let buy_count = transactions.iter().filter(|t| matches!(t.transaction_type, TransactionType::Buy)).count();
                            @let sell_count = transactions.iter().filter(|t| matches!(t.transaction_type, TransactionType::Sell)).count();

                            div class="" {
                                table class="" {
                                    thead class="" {
                                        tr {
                                            th class="" { "Date" }
                                            th class="" { "Type" }
                                            th class="" { "Symbol" }
                                            th class="" { "Shares" }
                                            th class="" { "Price" }
                                            th class="" { "Total" }
                                            th class="" { "Account" }
                                            th class="" { "Fees" }
                                        }
                                    }
                                    tbody class="" {
                                        @for transaction in transactions {
                                            tr class="" {
                                                td class="" {
                                                    (transaction.transaction_date)
                                                }
                                                td class="" {
                                                    @match transaction.transaction_type {
                                                        TransactionType::Buy => {
                                                            span class="" {
                                                                "Buy"
                                                            }
                                                        }
                                                        TransactionType::Sell => {
                                                            span class="" {
                                                                "Sell"
                                                            }
                                                        }
                                                    }
                                                }
                                                td class="" {
                                                    (transaction.ticker_symbol)
                                                }
                                                td class="" {
                                                    (format!("{:.4}", transaction.share_quantity))
                                                }
                                                td class="" {
                                                    (format!("{:.2} {}", transaction.price_per_share, transaction.currency))
                                                }
                                                td class="" {
                                                    (format!("{:.2} {}",
                                                        transaction.share_quantity * transaction.price_per_share + transaction.fees,
                                                        transaction.currency))
                                                }
                                                td class="" {
                                                    @let account_name = accounts.iter()
                                                        .find(|acc| acc.id == Some(transaction.account_id))
                                                        .map(|acc| acc.name.as_str())
                                                        .unwrap_or("Unknown Account");
                                                    (account_name)
                                                }
                                                td class="" {
                                                    @if transaction.fees > 0.0 {
                                                        (format!("{:.2} {}", transaction.fees, transaction.currency))
                                                    } @else {
                                                        "-"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Summary section
                            div class="" {
                                div class="" {
                                    h4 class="" { "Total Transactions" }
                                    p class="" { (total_transactions) }
                                }
                                div class="" {
                                    h4 class="" { "Buy Orders" }
                                    p class="" { (buy_count) }
                                }
                                div class="" {
                                    h4 class="" { "Sell Orders" }
                                    p class="" { (sell_count) }
                                }
                            }


                        }
                    }
                }
            })
        )
    }
}

#[get("/transactions")]
pub async fn list_all(
    db_state: &State<CoreState>,
    auth_user: CookieAuthenticatedUser<'_>,
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
    ))
}

#[get("/transactions/<account_id>")]
pub async fn list_by_account(
    account_id: AccountId,
    db_state: &State<CoreState>,
    auth_user: CookieAuthenticatedUser<'_>,
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
    ))
}
