use maud::{Markup, html};
use rocket::{
    State,
    form::{Form, FromForm},
    http::Status,
    request::FlashMessage,
    response::{Flash, Redirect},
};
use serde::{Deserialize, Serialize};

use crate::{
    core::{
        Account, AccountId, TransactionType,
        data_sources::{currencies, market_provider::SymbolSearchResult},
        state::CoreState,
        transactions::{CreateTransactionError, TransactionData, create_transaction},
    },
    site::{
        auth_guard::CookieAuthenticatedUser,
        shared::{NavSection, Shell},
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CreateTransactionFormError {
    TickerNotFound {
        symbol: String,
        suggestions: Vec<SymbolSearchResult>,
        form_data: TransactionForm,
    },
}

impl CreateTransactionFormError {
    pub fn to_flash_message(&self) -> String {
        let json = serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string());
        format!("CREATE_ERROR:{}", json)
    }

    pub fn from_flash_message(message: &str) -> Option<Self> {
        if let Some(json_str) = message.strip_prefix("CREATE_ERROR:") {
            serde_json::from_str(json_str).ok()
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromForm)]
pub struct TransactionForm {
    pub account_id: AccountId,
    pub transaction_type: String, // "buy" or "sell"
    pub ticker_symbol: String,
    pub share_quantity: f64,
    pub price_per_share: f64,
    pub currency: String,
    pub fees: f64,
    pub date: String, // YYYY-MM-DD format
}

/// Renders the transaction creation page.
#[get("/transactions/create")]
pub async fn create_page(
    db_state: &State<CoreState>,
    auth_user: CookieAuthenticatedUser<'_>,
    flash: Option<FlashMessage<'_>>,
) -> Result<Markup, Status> {
    let accounts = Account::for_user(&db_state.pool, auth_user.user_id)
        .await
        .map_err(|_| Status::InternalServerError)?;

    let form_data = if let Some(ref flash) = flash {
        CreateTransactionFormError::from_flash_message(flash.message()).and_then(
            |error| match error {
                CreateTransactionFormError::TickerNotFound { form_data, .. } => Some(form_data),
            },
        )
    } else {
        None
    };

    Ok(html! {
        (
            Shell::create(NavSection::Transactions, "Add Transaction", html! {
                form-container {
                    page-header {
                        page-header-title { "Add New Transaction" }
                        page-header-subtitle { "Record a new stock transaction" }
                    }

                    @match flash {
                        Some(flash) => {
                            @if let Some(structured_error) = CreateTransactionFormError::from_flash_message(flash.message()) {
                                @match structured_error {
                                    CreateTransactionFormError::TickerNotFound { symbol, suggestions, .. } => {
                                        alert data-alert-type="warning" {
                                            p {
                                                strong { "Error: " }
                                                "Ticker symbol '"
                                                (symbol)
                                                "' not found."
                                            }

                                            @if !suggestions.is_empty() {
                                                h4 class="suggestion-title" { "Did you mean one of these?" }

                                                suggestions-list {
                                                    @for suggestion in &suggestions {
                                                        suggestion-item {
                                                            suggestion-symbol { (suggestion.symbol) }
                                                            suggestion-name { (suggestion.name) }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            } @else {
                                // Regular error message
                                alert data-alert-type="warning" {
                                    p {
                                        strong { "Error: " }
                                        (flash.message())
                                    }
                                }
                            }
                        }
                        None => { /* No flash message to display */ }
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
                                                @let account_id = account.id.unwrap_or(0);
                                                @let is_selected = form_data
                                                    .as_ref()
                                                    .map_or(false, |data| data.account_id == account_id);
                                                @if is_selected {
                                                    option value=(account_id) selected { (account.name) }
                                                } @else {
                                                    option value=(account_id) { (account.name) }
                                                }
                                            }
                                        }
                                }
                            }

                            form-field {
                                label { "Transaction Type" }
                                radio-group {
                                    @let buy_checked = form_data
                                        .as_ref()
                                        .map_or(false, |data| data.transaction_type == "buy");
                                    @let sell_checked = form_data
                                        .as_ref()
                                        .map_or(false, |data| data.transaction_type == "sell");

                                    radio-option class="buy-option" {
                                        @if buy_checked {
                                            input type="radio" id="buy" name="transaction_type" value="buy" required checked;
                                        } @else {
                                            input type="radio" id="buy" name="transaction_type" value="buy" required;
                                        }
                                        label for="buy" {
                                            "Buy"
                                        }
                                    }
                                    radio-option class="sell-option" {
                                        @if sell_checked {
                                            input type="radio" id="sell" name="transaction_type" value="sell" required checked;
                                        } @else {
                                            input type="radio" id="sell" name="transaction_type" value="sell" required;
                                        }
                                        label for="sell" {
                                            "Sell"
                                        }
                                    }
                                }
                            }

                            form-field {
                                label for="ticker_symbol" { "Ticker Symbol" }
                                @let ticker_value = form_data
                                    .as_ref()
                                    .map_or("", |data| &data.ticker_symbol);
                                input
                                    type="text"
                                    id="ticker_symbol"
                                    name="ticker_symbol"
                                    required
                                    value=(ticker_value)
                                    placeholder="e.g., AAPL, MSFT";
                            }

                            form-row {
                                form-field {
                                    label for="share_quantity" { "Shares" }
                                    @let shares_value = form_data
                                        .as_ref()
                                        .map_or(String::new(), |data| data.share_quantity.to_string());
                                    input
                                        type="number"
                                        id="share_quantity"
                                        name="share_quantity"
                                        step="0.000000001"
                                        min="0"
                                        required
                                        value=(shares_value)
                                        placeholder="10";
                                }

                                form-field {
                                    label for="price_per_share" { "Price per Share" }
                                    @let price_value = form_data
                                        .as_ref()
                                        .map_or(String::new(), |data| data.price_per_share.to_string());
                                    input
                                        type="number"
                                        id="price_per_share"
                                        name="price_per_share"
                                        step="0.00001"
                                        min="0"
                                        required
                                        value=(price_value)
                                        placeholder="150.00";
                                }
                            }

                            form-row {
                                form-field {
                                    label for="currency" { "Currency" }
                                    @let selected_currency = form_data
                                        .as_ref()
                                        .map_or("EUR", |data| &data.currency);
                                    select
                                        id="currency"
                                        name="currency"
                                        required {
                                            @for code in currencies::get_currency_codes() {
                                                @if code == selected_currency {
                                                    option value=(code) selected { (code) }
                                                } @else {
                                                    option value=(code) { (code) }
                                                }
                                            }
                                        }
                                }

                                form-field {
                                    label for="fees" { "Fees" }
                                    @let fees_value = form_data
                                        .as_ref()
                                        .map_or("0".to_string(), |data| data.fees.to_string());
                                    input
                                        type="number"
                                        id="fees"
                                        name="fees"
                                        step="0.01"
                                        min="0"
                                        value=(fees_value)
                                        placeholder="0.00";
                                }
                            }

                            form-field {
                                label for="date" { "Transaction Date" }
                                @let date_value = form_data
                                    .as_ref()
                                    .map_or("", |data| &data.date);
                                input
                                    type="date"
                                    id="date"
                                    name="date"
                                    value=(date_value)
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
) -> Result<Redirect, Flash<Redirect>> {
    // Convert string transaction type to enum
    let transaction_type = match form.transaction_type.as_str() {
        "buy" => TransactionType::Buy,
        "sell" => TransactionType::Sell,
        _ => {
            return Err(Flash::error(
                Redirect::to("/transactions/create"),
                "Invalid transaction type selected.",
            ));
        }
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

    // Create the transaction with market provider validation
    match create_transaction(
        &db_state.pool,
        db_state.market_provider.as_ref(),
        auth_user.user_id,
        &transaction_data,
    )
    .await
    {
        Ok(_) => {
            // Redirect to transactions list on success
            Ok(Redirect::to("/transactions"))
        }
        Err(CreateTransactionError::TickerSymbolNotFound {
            symbol,
            suggestions,
        }) => {
            // Create a structured error with form data preservation
            let structured_error = CreateTransactionFormError::TickerNotFound {
                symbol,
                suggestions: suggestions.into_iter().take(3).collect(),
                form_data: form.into_inner(),
            };

            Err(Flash::error(
                Redirect::to("/transactions/create"),
                structured_error.to_flash_message(),
            ))
        }
        Err(CreateTransactionError::InvalidPricePerShare) => Err(Flash::error(
            Redirect::to("/transactions/create"),
            "Price per share must be greater than 0.",
        )),
        Err(CreateTransactionError::InvalidShareQuantity) => Err(Flash::error(
            Redirect::to("/transactions/create"),
            "Share quantity must be greater than 0.",
        )),
        Err(CreateTransactionError::InvalidCurrency) => Err(Flash::error(
            Redirect::to("/transactions/create"),
            "Invalid currency selected.",
        )),
        Err(CreateTransactionError::InvalidTickerSymbol) => Err(Flash::error(
            Redirect::to("/transactions/create"),
            "Ticker symbol cannot be empty.",
        )),
        Err(CreateTransactionError::InvalidDate) => Err(Flash::error(
            Redirect::to("/transactions/create"),
            "Invalid date format. Please use YYYY-MM-DD.",
        )),
        Err(CreateTransactionError::AccountNotFound) => Err(Flash::error(
            Redirect::to("/transactions/create"),
            "The selected account was not found.",
        )),
        Err(CreateTransactionError::MarketDataUnavailable) => Err(Flash::error(
            Redirect::to("/transactions/create"),
            "Market data service is currently unavailable. Please try again later.",
        )),
        Err(CreateTransactionError::DatabaseError(_)) => Err(Flash::error(
            Redirect::to("/transactions/create"),
            "A database error occurred. Please try again.",
        )),
    }
}
