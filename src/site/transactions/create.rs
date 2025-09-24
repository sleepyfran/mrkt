use maud::{Markup, html};
use rocket::{
    State,
    form::{Contextual, Form, FromForm},
    http::Status,
    response::Redirect,
};

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

#[derive(Debug, Clone, FromForm)]
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
) -> Result<Markup, Status> {
    let accounts = Account::for_user(&db_state.pool, auth_user.user_id)
        .await
        .map_err(|_| Status::InternalServerError)?;

    let empty_context = Contextual {
        value: None,
        context: rocket::form::Context::default(),
    };

    render_form(accounts, &empty_context)
}

fn render_form(
    accounts: Vec<Account>,
    form_context: &Contextual<TransactionForm>,
) -> Result<Markup, Status> {
    render_form_internal(accounts, form_context, None)
}

fn render_form_with_ticker_error(
    accounts: Vec<Account>,
    form_context: &Contextual<TransactionForm>,
    invalid_symbol: &str,
    suggestions: &[SymbolSearchResult],
) -> Result<Markup, Status> {
    render_form_internal(accounts, form_context, Some((invalid_symbol, suggestions)))
}

fn render_form_internal(
    accounts: Vec<Account>,
    form_context: &Contextual<TransactionForm>,
    ticker_error: Option<(&str, &[SymbolSearchResult])>,
) -> Result<Markup, Status> {
    Ok(html! {
        (
            Shell::create(NavSection::Transactions, "Add Transaction", html! {
                form-container {
                    page-header {
                        page-header-title { "Add New Transaction" }
                        page-header-subtitle { "Record a new stock transaction" }
                    }

                    @let has_errors = form_context.context.errors().count() > 0;
                    @if has_errors {
                        alert data-alert-type="error" {
                            p {
                                strong { "Please fix the following errors:" }
                            }
                            ul {
                                @for error in form_context.context.errors() {
                                    li { (error) }
                                }
                            }
                        }
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
                                            option value="" disabled { "Select an account" }
                                            @for account in accounts {
                                                @let account_id = account.id.unwrap_or(0);
                                                @let is_selected = form_context.context.field_value("account_id")
                                                    .map_or(false, |val| val == account_id.to_string());
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
                                    @let transaction_type_value = form_context.context.field_value("transaction_type")
                                        .unwrap_or("");
                                    @let buy_checked = transaction_type_value == "buy";
                                    @let sell_checked = transaction_type_value == "sell";

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
                                @let ticker_value = form_context.context.field_value("ticker_symbol")
                                    .unwrap_or("");
                                input
                                    type="text"
                                    id="ticker_symbol"
                                    name="ticker_symbol"
                                    required
                                    value=(ticker_value)
                                    placeholder="e.g., AAPL, MSFT";

                                p class="suggestion" { "You can also type a name to get suggestions once you submit the form."}
                            }

                            @if let Some((invalid_symbol, suggestions)) = ticker_error {
                                alert data-alert-type="warning" {
                                    p {
                                        strong { "Error: " }
                                        "Ticker symbol '"
                                        (invalid_symbol)
                                        "' not found."
                                    }

                                    @if !suggestions.is_empty() {
                                        h4 class="suggestion-title" { "Did you mean one of these?" }

                                        suggestions-list {
                                            @for suggestion in suggestions.iter().take(3) {
                                                suggestion-item {
                                                    suggestion-symbol { (suggestion.symbol) }
                                                    suggestion-name { (suggestion.name) }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            form-row {
                                form-field {
                                    label for="share_quantity" { "Shares" }
                                    @let shares_value = form_context.context.field_value("share_quantity")
                                        .unwrap_or("");
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
                                    @let price_value = form_context.context.field_value("price_per_share")
                                        .unwrap_or("");
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
                                    @let selected_currency = form_context.context.field_value("currency")
                                        .unwrap_or("EUR");
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
                                    @let fees_value = form_context.context.field_value("fees")
                                        .unwrap_or("0");
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
                                @let date_value = form_context.context.field_value("date")
                                    .unwrap_or("");
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
              .add_script("ticker-suggestions.js")
        )
    })
}

/// Handler for transaction creation form submission.
#[post("/transactions/create", data = "<form>")]
pub async fn create_submit(
    form: Form<Contextual<'_, TransactionForm>>,
    db_state: &State<CoreState>,
    auth_user: CookieAuthenticatedUser<'_>,
) -> Result<Redirect, (Status, Markup)> {
    let form_data = match &form.value {
        Some(data) => data,
        None => {
            // Form has validation errors, render the form page with errors.
            let accounts = match Account::for_user(&db_state.pool, auth_user.user_id).await {
                Ok(accounts) => accounts,
                Err(_) => return Err((Status::InternalServerError, html! { "Database error" })),
            };
            match render_form(accounts, &form) {
                Ok(markup) => return Err((Status::UnprocessableEntity, markup)),
                Err(_) => return Err((Status::InternalServerError, html! { "Template error" })),
            }
        }
    };

    let transaction_type = match form_data.transaction_type.as_str() {
        "buy" => TransactionType::Buy,
        "sell" => TransactionType::Sell,
        _ => {
            // Couldn't parse transaction type, re-render with errors.
            let accounts = match Account::for_user(&db_state.pool, auth_user.user_id).await {
                Ok(accounts) => accounts,
                Err(_) => return Err((Status::InternalServerError, html! { "Database error" })),
            };
            match render_form(accounts, &form) {
                Ok(markup) => return Err((Status::UnprocessableEntity, markup)),
                Err(_) => return Err((Status::InternalServerError, html! { "Template error" })),
            }
        }
    };

    let transaction_data = TransactionData {
        account_id: form_data.account_id,
        transaction_type,
        price_per_share: form_data.price_per_share,
        share_quantity: form_data.share_quantity,
        fees: form_data.fees,
        currency: form_data.currency.clone(),
        ticker_symbol: form_data.ticker_symbol.clone(),
        date: form_data.date.clone(),
    };

    match create_transaction(
        &db_state.pool,
        db_state.market_provider.as_ref(),
        auth_user.user_id,
        &transaction_data,
    )
    .await
    {
        Ok(_) => Ok(Redirect::to("/transactions")),
        Err(error) => {
            let accounts = match Account::for_user(&db_state.pool, auth_user.user_id).await {
                Ok(accounts) => accounts,
                Err(_) => return Err((Status::InternalServerError, html! { "Database error" })),
            };

            let mut error_form = form.into_inner();

            match error {
                CreateTransactionError::TickerSymbolNotFound {
                    symbol,
                    suggestions,
                } => {
                    match render_form_with_ticker_error(
                        accounts,
                        &error_form,
                        &symbol,
                        &suggestions,
                    ) {
                        Ok(markup) => return Err((Status::UnprocessableEntity, markup)),
                        Err(_) => {
                            return Err((Status::InternalServerError, html! { "Template error" }));
                        }
                    }
                }
                _ => {
                    let error_message = match error {
                        CreateTransactionError::InvalidPricePerShare => {
                            "Price per share must be greater than 0."
                        }
                        CreateTransactionError::InvalidShareQuantity => {
                            "Share quantity must be greater than 0."
                        }
                        CreateTransactionError::InvalidCurrency => "Invalid currency selected.",
                        CreateTransactionError::InvalidTickerSymbol => {
                            "Ticker symbol cannot be empty."
                        }
                        CreateTransactionError::InvalidDate => {
                            "Invalid date format. Please use YYYY-MM-DD."
                        }
                        CreateTransactionError::AccountNotFound => {
                            "The selected account was not found."
                        }
                        CreateTransactionError::MarketDataUnavailable => {
                            "Market data service is currently unavailable. Please try again later."
                        }
                        CreateTransactionError::DatabaseError(_) => {
                            "A database error occurred. Please try again."
                        }
                        CreateTransactionError::TickerSymbolNotFound { .. } => unreachable!(),
                    };

                    error_form
                        .context
                        .push_error(rocket::form::Error::validation(error_message));

                    match render_form(accounts, &error_form) {
                        Ok(markup) => return Err((Status::UnprocessableEntity, markup)),
                        Err(_) => {
                            return Err((Status::InternalServerError, html! { "Template error" }));
                        }
                    }
                }
            }
        }
    }
}
