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
    site::{auth_guard::CookieAuthenticatedUser, shared::base_template},
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
        (base_template())
        body class="bg-gray-50 min-h-screen flex items-center justify-center py-12 px-4 sm:px-6 lg:px-8" {
            div class="max-w-md w-full space-y-8" {
                div class="text-center" {
                    h1 class="text-3xl font-bold text-gray-900 mb-2" { "Add New Transaction" }
                    p class="text-sm text-gray-600" { "Record a new stock transaction" }
                }

                form method="post" action="/transactions/create" class="mt-8 space-y-6 bg-white p-8 rounded-lg shadow-md" {
                    div class="space-y-4" {
                        div {
                            label for="account_id" class="block text-sm font-medium text-gray-700 mb-1" { "Account" }
                            select
                                id="account_id"
                                name="account_id"
                                required
                                class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition duration-200" {
                                    option value="" disabled selected { "Select an account" }
                                    @for account in accounts {
                                        option value=(account.id.unwrap_or(0)) { (account.name) }
                                    }
                                }
                        }

                        div {
                            label class="block text-sm font-medium text-gray-700 mb-1" { "Transaction Type" }
                            div class="flex w-full rounded-md shadow-sm" role="group" {
                                input type="radio" id="buy" name="transaction_type" value="buy" required class="sr-only peer/buy";
                                label for="buy" class="flex-1 px-4 py-2 text-sm font-medium text-gray-900 bg-white border border-gray-300 rounded-l-md hover:bg-gray-50 focus:z-10 focus:ring-2 focus:ring-blue-500 peer-checked/buy:bg-blue-600 peer-checked/buy:text-white peer-checked/buy:border-blue-600 peer-checked/buy:hover:bg-blue-700 cursor-pointer transition duration-200 text-center" {
                                    "Buy"
                                }
                                input type="radio" id="sell" name="transaction_type" value="sell" required class="sr-only peer/sell";
                                label for="sell" class="flex-1 px-4 py-2 text-sm font-medium text-gray-900 bg-white border border-gray-300 rounded-r-md hover:bg-gray-50 focus:z-10 focus:ring-2 focus:ring-blue-500 peer-checked/sell:bg-red-600 peer-checked/sell:text-white peer-checked/sell:border-red-600 peer-checked/sell:hover:bg-red-700 cursor-pointer transition duration-200 border-l-0 text-center" {
                                    "Sell"
                                }
                            }
                        }

                        div {
                            label for="ticker_symbol" class="block text-sm font-medium text-gray-700 mb-1" { "Ticker Symbol" }
                            input
                                type="text"
                                id="ticker_symbol"
                                name="ticker_symbol"
                                required
                                class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition duration-200"
                                placeholder="e.g., AAPL, MSFT";
                        }

                        div class="grid grid-cols-2 gap-4" {
                            div {
                                label for="share_quantity" class="block text-sm font-medium text-gray-700 mb-1" { "Shares" }
                                input
                                    type="number"
                                    id="share_quantity"
                                    name="share_quantity"
                                    step="0.0001"
                                    min="0"
                                    required
                                    class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition duration-200"
                                    placeholder="10";
                            }

                            div {
                                label for="price_per_share" class="block text-sm font-medium text-gray-700 mb-1" { "Price per Share" }
                                input
                                    type="number"
                                    id="price_per_share"
                                    name="price_per_share"
                                    step="0.01"
                                    min="0"
                                    required
                                    class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition duration-200"
                                    placeholder="150.00";
                            }
                        }

                        div class="grid grid-cols-2 gap-4" {
                            div {
                                label for="currency" class="block text-sm font-medium text-gray-700 mb-1" { "Currency" }
                                input
                                    type="text"
                                    id="currency"
                                    name="currency"
                                    value="USD"
                                    required
                                    class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition duration-200"
                                    placeholder="USD";
                            }

                            div {
                                label for="fees" class="block text-sm font-medium text-gray-700 mb-1" { "Fees" }
                                input
                                    type="number"
                                    id="fees"
                                    name="fees"
                                    step="0.01"
                                    min="0"
                                    value="0"
                                    class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition duration-200"
                                    placeholder="0.00";
                            }
                        }

                        div {
                            label for="date" class="block text-sm font-medium text-gray-700 mb-1" { "Transaction Date" }
                            input
                                type="date"
                                id="date"
                                name="date"
                                required
                                class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition duration-200";
                        }
                    }

                    div class="pt-4" {
                        input
                            type="submit"
                            value="Add Transaction"
                            class="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 cursor-pointer transition duration-200";
                    }
                }

                div class="text-center mt-4" {
                    a href="/transactions" class="font-medium text-blue-600 hover:text-blue-500" {
                        "← Back to transactions"
                    }
                }
            }
        }
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
