use maud::{Markup, html, Render};
use rocket::{get, post, State, form::Form, request::FlashMessage, http::Status};
use rocket::fs::TempFile;

use crate::{
    core::{
        state::CoreState,
        Account, AccountId,
        import::{ImportProviderTrait, ImportResult, ImportError, PortfolioPerformanceProvider, read_file_contents},
    },
    site::{
        auth_guard::CookieAuthenticatedUser,
        shared::{NavSection, Shell},
    },
};

#[derive(FromForm)]
pub struct ImportForm<'f> {
    pub file: TempFile<'f>,
    pub account_id: AccountId,
}

/// Get all available import providers
fn get_available_providers() -> Vec<Box<dyn ImportProviderTrait + Send + Sync>> {
    vec![
        Box::new(PortfolioPerformanceProvider::new()),
    ]
}

pub fn routes() -> Vec<rocket::Route> {
    routes![import_selection_page, import_page, import_submit]
}

/// Renders the import provider selection page.
#[get("/configuration/import")]
pub async fn import_selection_page(
    _db_state: &State<CoreState>,
    _auth_user: CookieAuthenticatedUser<'_>,
    flash: Option<FlashMessage<'_>>,
) -> Markup {
    let providers = get_available_providers();
    let provider_infos: Vec<_> = providers.iter().map(|p| p.provider_info()).collect();

    Shell::create(NavSection::Configuration, "Import Data", html! {
        link rel="stylesheet" href="/static/configuration.css";
        
        page-header {
            page-header-title { "Import Transaction Data" }
            page-header-subtitle { "Choose the source of your transaction data" }
        }

        @if let Some(ref flash_msg) = flash {
            @if flash_msg.kind() == "error" {
                form-container {
                    alert data-alert-type="error" {
                        p { (flash_msg.message()) }
                    }
                }
            }
        }

        configuration-sections {
            configuration-section {
                h3 { "Available Import Sources" }
                p { "Select the application or service you want to import transaction data from:" }
                
                configuration-cards {
                    @for provider in &provider_infos {
                        configuration-card {
                            configuration-card-header {
                                h4 { (provider.name) }
                                p { (provider.description) }
                            }
                            
                            configuration-card-details {
                                p {
                                    strong { "Supported file types: " }
                                    (provider.supported_file_types.join(", ").to_uppercase())
                                }
                            }
                            
                            configuration-card-actions {
                                a href=(format!("/configuration/import/{}", provider.id)) class="button button-primary" {
                                    "Import from " (provider.name)
                                }
                            }
                        }
                    }
                }
            }
        }

        div {
            a href="/configuration" class="back-link" {
                "← Back to configuration"
            }
        }
    })
    .add_stylesheet("configuration.css")
    .render()
}

/// Renders the provider-specific import page.
#[get("/configuration/import/<provider_id>")]
pub async fn import_page(
    provider_id: String,
    db_state: &State<CoreState>,
    auth_user: CookieAuthenticatedUser<'_>,
    flash: Option<FlashMessage<'_>>,
) -> Result<Markup, Status> {
    // Validate that the provider exists.
    let providers = get_available_providers();
    let provider_info = providers.iter()
        .map(|p| p.provider_info())
        .find(|p| p.id == provider_id)
        .ok_or(Status::NotFound)?;

    let accounts = Account::for_user(&db_state.pool, auth_user.user_id)
        .await
        .map_err(|_| Status::InternalServerError)?;

    render_provider_import_form(provider_info, accounts, None, flash)
}

fn render_provider_import_form(
    provider_info: crate::core::import::ImportProvider,
    accounts: Vec<Account>,
    import_result: Option<&ImportResult>,
    flash: Option<FlashMessage<'_>>,
) -> Result<Markup, Status> {
    let accept_types = provider_info.supported_file_types.iter()
        .map(|ext| format!(".{}", ext))
        .collect::<Vec<_>>()
        .join(",");

    Ok(Shell::create(NavSection::Configuration, &format!("Import from {}", provider_info.name), html! {
        link rel="stylesheet" href="/static/configuration.css";
        
        page-header {
            page-header-title { "Import from " (provider_info.name) }
            page-header-subtitle { (provider_info.description) }
        }

        form-container {
            // Display import results if available.
            @if let Some(result) = import_result {
                @if result.failed == 0 {
                    alert data-alert-type="success" {
                        p {
                            "Successfully imported " (result.successful) " transactions."
                        }
                        @if result.successful > 0 {
                            a href="/transactions" { "View imported transactions" }
                        }
                    }
                } @else {
                    alert data-alert-type="warning" {
                        p {
                            strong { "Import completed with issues: " }
                            (result.successful) " successful, " (result.failed) " failed transactions."
                        }
                    }

                    @if !result.errors.is_empty() {
                        alert data-alert-type="error" {
                            p {
                                strong { "Failed transactions:" }
                            }
                            ul {
                                @for error in &result.errors {
                                    li {
                                        "Row " (error.row_number) ": " (error.error)
                                        @if !error.raw_data.is_empty() {
                                            br;
                                            small {
                                                @for (key, value) in &error.raw_data {
                                                    (key) ": " (value) " "
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
            @if accounts.is_empty() {
                alert data-alert-type="warning" {
                    p { "You don't have any accounts yet. You need to create at least one account before importing transactions." }
                    a href="/accounts/create" {
                        "Create Your First Account"
                    }
                }
            } @else {
                form method="post" action=(format!("/configuration/import/{}", provider_info.id)) class="form-card" enctype="multipart/form-data" {
                    // Hidden input for provider ID.
                    input type="hidden" name="provider_id" value=(provider_info.id);

                    form-grid {
                        form-field {
                            label for="account_id" { "Target Account" }
                            select id="account_id" name="account_id" required {
                                option value="" disabled selected { "Select account to import into" }
                                @for account in accounts {
                                    @let account_id = account.id.unwrap_or(0);
                                    option value=(account_id) { (account.name) }
                                }
                            }
                            p class="field-help" { "All imported transactions will be added to this account" }
                        }

                        form-field {
                            label for="file" { "Data File" }
                            input type="file" id="file" name="file" accept=(accept_types) required;
                            p class="field-help" { 
                                "Supported file types: " (provider_info.supported_file_types.join(", ").to_uppercase())
                            }
                        }
                    }

                    form-submit {
                        input type="submit" value="Import Transactions";
                    }
                }

                import-info {
                    h3 { "About " (provider_info.name) " Import" }
                    p { (provider_info.description) }

                    @match provider_info.id.as_str() {
                        "portfolio-performance" => {
                            h4 { "Supported Transaction Types" }
                            ul {
                                li { strong { "Buy" } " - Purchase transactions" }
                                li { strong { "Sell" } " - Sale transactions" }
                                li { strong { "Delivery (Inbound)" } " - Stock transfers or grants (imported as transfers)" }
                            }
                            
                            h4 { "Requirements" }
                            ul {
                                li { "File must be in CSV format" }
                                li { "First row must contain headers" }
                                li { "Required columns: Date, Type, Ticker Symbol, Shares, Currency" }
                                li { "Dates should be in ISO format (YYYY-MM-DD)" }
                            }
                        }
                        _ => { /* No additional info for other providers yet */ }
                    }
                }
            }
        }

        div {
            a href="/configuration/import" class="back-link" {
                "← Back to import selection"
            }
        }
    }).attach_flash(flash).render())
}

/// Handler for CSV import form submission.
#[post("/configuration/import/<provider_id>", data = "<form>")]
pub async fn import_submit(
    provider_id: String,
    form: Form<ImportForm<'_>>,
    db_state: &State<CoreState>,
    auth_user: CookieAuthenticatedUser<'_>,
) -> Result<Markup, (Status, Markup)> {
    // Find the selected provider using the URL parameter
    let providers = get_available_providers();
    let provider = providers.iter()
        .find(|p| p.provider_info().id == provider_id);
    
    let (provider, provider_info) = match provider {
        Some(p) => {
            let info = p.provider_info();
            (p, info)
        },
        None => {
            let accounts = Account::for_user(&db_state.pool, auth_user.user_id)
                .await
                .map_err(|_| (Status::InternalServerError, html! { "Database error" }))?;
            
            let error_result = ImportResult {
                successful: 0,
                failed: 1,
                errors: vec![ImportError {
                    row_number: 0,
                    error: "Invalid import provider selected".to_string(),
                    raw_data: std::collections::HashMap::new(),
                }],
            };
            
            // Create a dummy provider info for error rendering
            let dummy_provider = crate::core::import::ImportProvider {
                id: provider_id,
                name: "Unknown Provider".to_string(),
                description: "Provider not found".to_string(),
                supported_file_types: vec![],
            };
            
            return render_provider_import_form(dummy_provider, accounts, Some(&error_result), None)
                .map_err(|status| (status, html! { "Template error" }));
        }
    };

    // Read the file contents.
    let file_contents = match form.file.open().await {
        Ok(stream) => {
            match read_file_contents(stream).await {
                Ok(contents) => contents,
                Err(_) => {
                    let accounts = Account::for_user(&db_state.pool, auth_user.user_id)
                        .await
                        .map_err(|_| (Status::InternalServerError, html! { "Database error" }))?;
                    
                    let error_result = ImportResult {
                        successful: 0,
                        failed: 1,
                        errors: vec![ImportError {
                            row_number: 0,
                            error: "Failed to read uploaded file".to_string(),
                            raw_data: std::collections::HashMap::new(),
                        }],
                    };
                    return render_provider_import_form(provider_info, accounts, Some(&error_result), None)
                        .map_err(|status| (status, html! { "Template error" }));
                }
            }
        }
        Err(_) => {
            let accounts = Account::for_user(&db_state.pool, auth_user.user_id)
                .await
                .map_err(|_| (Status::InternalServerError, html! { "Database error" }))?;
            
            let error_result = ImportResult {
                successful: 0,
                failed: 1,
                errors: vec![ImportError {
                    row_number: 0,
                    error: "Failed to open uploaded file".to_string(),
                    raw_data: std::collections::HashMap::new(),
                }],
            };
            return render_provider_import_form(provider_info, accounts, Some(&error_result), None)
                .map_err(|status| (status, html! { "Template error" }));
        }
    };

    let import_result = provider.import(
        &db_state.pool,
        db_state.market_provider.as_ref(),
        auth_user.user_id,
        form.account_id,
        &file_contents,
    ).await;

    let accounts = Account::for_user(&db_state.pool, auth_user.user_id)
        .await
        .map_err(|_| (Status::InternalServerError, html! { "Database error" }))?;

    match import_result {
        Ok(result) => {
            render_provider_import_form(provider_info, accounts, Some(&result), None)
                .map_err(|status| (status, html! { "Template error" }))
        }
        Err(_) => {
            let error_result = ImportResult {
                successful: 0,
                failed: 1,
                errors: vec![ImportError {
                    row_number: 0,
                    error: "File format error or invalid data structure".to_string(),
                    raw_data: std::collections::HashMap::new(),
                }],
            };
            render_provider_import_form(provider_info, accounts, Some(&error_result), None)
                .map_err(|status| (status, html! { "Template error" }))
        }
    }
}