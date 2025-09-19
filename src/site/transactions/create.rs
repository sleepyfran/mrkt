use maud::{Markup, html};
use rocket::{
    State,
    form::{Form, FromForm},
    http::Status,
    response::Redirect,
};

use crate::{
    core::{
        Account, AccountId, TransactionType,
        data_sources::currencies,
        state::CoreState,
        transactions::{CreateTransactionError, TransactionData, create_transaction},
    },
    site::{
        auth_guard::CookieAuthenticatedUser,
        shared::{NavSection, Shell},
    },
};

#[derive(FromForm)]
pub struct TransactionForm {
    account_id: AccountId,
    transaction_type: String, // "buy" or "sell"
    ticker_symbol: String,
    share_quantity: f64,
    price_per_share: f64,
    currency: String,
    fees: f64,
    date: String, // YYYY-MM-DD format
}

/// Renders the transaction creation page.
#[get("/transactions/create")]
pub async fn create_page(
    db_state: &State<CoreState>,
    auth_user: CookieAuthenticatedUser<'_>,
) -> Result<Markup, Status> {
    // Get user's accounts for the dropdown
    let accounts = Account::for_user(&db_state.pool, auth_user.user_id)
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(html! {
        (
            Shell::create(NavSection::Transactions, "Add Transaction", html! {
                form-container {
                    page-header {
                        page-header-title { "Add New Transaction" }
                        page-header-subtitle { "Record a new stock transaction" }
                    }

                    form method="post" action="/transactions/create" class="form-card" {
                        form-grid {
                            form-field {
                                label for="account_id" { "Account" }
                                @if accounts.is_empty() {
                                    alert data-alert-type="warning" {
                                        p { "You don't have any accounts yet." }
                                        a href="/accounts/create" {
                                            "Create Your First Account"
                                        }
                                    }
                                } @else {
                                    select
                                        id="account_id"
                                        name="account_id"
                                        required {
                                            option value="" disabled selected { "Select an account" }
                                            @for account in accounts {
                                                option value=(account.id.unwrap_or(0)) { (account.name) }
                                            }
                                        }
                                }
                            }

                            form-field {
                                label { "Transaction Type" }
                                radio-group {
                                    radio-option class="buy-option" {
                                        input type="radio" id="buy" name="transaction_type" value="buy" required;
                                        label for="buy" {
                                            "Buy"
                                        }
                                    }
                                    radio-option class="sell-option" {
                                        input type="radio" id="sell" name="transaction_type" value="sell" required;
                                        label for="sell" {
                                            "Sell"
                                        }
                                    }
                                }
                            }

                            form-field {
                                label for="ticker_symbol" { "Ticker Symbol" }
                                input
                                    type="text"
                                    id="ticker_symbol"
                                    name="ticker_symbol"
                                    required
                                    placeholder="e.g., AAPL, MSFT";
                            }

                            form-row {
                                form-field {
                                    label for="share_quantity" { "Shares" }
                                    input
                                        type="number"
                                        id="share_quantity"
                                        name="share_quantity"
                                        step="0.0001"
                                        min="0"
                                        required
                                        placeholder="10";
                                }

                                form-field {
                                    label for="price_per_share" { "Price per Share" }
                                    input
                                        type="number"
                                        id="price_per_share"
                                        name="price_per_share"
                                        step="0.01"
                                        min="0"
                                        required
                                        placeholder="150.00";
                                }
                            }

                            form-row {
                                form-field {
                                    label for="currency" { "Currency" }
                                    select
                                        id="currency"
                                        name="currency"
                                        required {
                                            @for code in currencies::get_currency_codes() {
                                                // TODO: Make configurable through an environment variable.
                                                @if code == "EUR" {
                                                    option value=(code) selected { (code) }
                                                } @else {
                                                    option value=(code) { (code) }
                                                }
                                            }
                                        }
                                }

                                form-field {
                                    label for="fees" { "Fees" }
                                    input
                                        type="number"
                                        id="fees"
                                        name="fees"
                                        step="0.01"
                                        min="0"
                                        value="0"
                                        placeholder="0.00";
                                }
                            }

                            form-field {
                                label for="date" { "Transaction Date" }
                                input
                                    type="date"
                                    id="date"
                                    name="date"
                                    required;
                            }
                        }

                        form-submit {
                            input
                                type="submit"
                                value="Add Transaction";
                        }
                    }

                    div {
                        a href="/transactions" class="back-link" {
                            "← Back to transactions"
                        }
                    }
                }
            }).add_stylesheet("transactions.css")
        )
    })
}

/// Handler for transaction creation form submission.
#[post("/transactions/create", data = "<form>")]
pub async fn create_submit(
    form: Form<TransactionForm>,
    db_state: &State<CoreState>,
    auth_user: CookieAuthenticatedUser<'_>,
) -> Result<Redirect, CreateTransactionError> {
    // Convert string transaction type to enum
    let transaction_type = match form.transaction_type.as_str() {
        "buy" => TransactionType::Buy,
        "sell" => TransactionType::Sell,
        _ => return Err(CreateTransactionError::InvalidTickerSymbol), // Using this error as a proxy for invalid input
    };

    // Create the transaction data
    let transaction_data = TransactionData {
        account_id: form.account_id,
        transaction_type,
        price_per_share: form.price_per_share,
        share_quantity: form.share_quantity,
        fees: form.fees,
        currency: form.currency.clone(),
        ticker_symbol: form.ticker_symbol.clone(),
        date: form.date.clone(),
    };

    // Create the transaction
    create_transaction(&db_state.pool, auth_user.user_id, &transaction_data).await?;

    // Redirect to transactions list on success
    Ok(Redirect::to("/transactions"))
}
