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
                div class="" {
                    div class="" {
                        h1 class="" { "Add New Transaction" }
                        p class="" { "Record a new stock transaction" }
                    }

                    form method="post" action="/transactions/create" class="" {
                        div class="" {
                            div {
                                label for="account_id" class="" { "Account" }
                                @if accounts.is_empty() {
                                    div class="" {
                                        p class="" { "You don't have any accounts yet." }
                                        a href="/accounts/create" class="" {
                                            "Create Your First Account"
                                        }
                                    }
                                } @else {
                                    select
                                        id="account_id"
                                        name="account_id"
                                        required
                                        class="" {
                                            option value="" disabled selected { "Select an account" }
                                            @for account in accounts {
                                                option value=(account.id.unwrap_or(0)) { (account.name) }
                                            }
                                        }
                                }
                            }

                            div {
                                label class="" { "Transaction Type" }
                                div class="" role="group" {
                                    input type="radio" id="buy" name="transaction_type" value="buy" required class="";
                                    label for="buy" class="" {
                                        "Buy"
                                    }
                                    input type="radio" id="sell" name="transaction_type" value="sell" required class="";
                                    label for="sell" class="" {
                                        "Sell"
                                    }
                                }
                            }

                            div {
                                label for="ticker_symbol" class="" { "Ticker Symbol" }
                                input
                                    type="text"
                                    id="ticker_symbol"
                                    name="ticker_symbol"
                                    required
                                    class=""
                                    placeholder="e.g., AAPL, MSFT";
                            }

                            div class="" {
                                div {
                                    label for="share_quantity" class="" { "Shares" }
                                    input
                                        type="number"
                                        id="share_quantity"
                                        name="share_quantity"
                                        step="0.0001"
                                        min="0"
                                        required
                                        class=""
                                        placeholder="10";
                                }

                                div {
                                    label for="price_per_share" class="" { "Price per Share" }
                                    input
                                        type="number"
                                        id="price_per_share"
                                        name="price_per_share"
                                        step="0.01"
                                        min="0"
                                        required
                                        class=""
                                        placeholder="150.00";
                                }
                            }

                            div class="" {
                                div {
                                    label for="currency" class="" { "Currency" }
                                    input
                                        type="text"
                                        id="currency"
                                        name="currency"
                                        value="USD"
                                        required
                                        class=""
                                        placeholder="USD";
                                }

                                div {
                                    label for="fees" class="" { "Fees" }
                                    input
                                        type="number"
                                        id="fees"
                                        name="fees"
                                        step="0.01"
                                        min="0"
                                        value="0"
                                        class=""
                                        placeholder="0.00";
                                }
                            }

                            div {
                                label for="date" class="" { "Transaction Date" }
                                input
                                    type="date"
                                    id="date"
                                    name="date"
                                    required
                                    class="";
                            }
                    }

                        div class="" {
                            input
                                type="submit"
                                value="Add Transaction"
                                class="";
                        }
                    }

                    div class="" {
                        a href="/transactions" class="" {
                            "← Back to transactions"
                        }
                    }
                }
            })
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
