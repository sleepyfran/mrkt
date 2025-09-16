use maud::{Markup, html};
use rocket::{State, http::Status};

use crate::{
    core::{
        state::CoreState,
        accounts::{ListAllAccountsError, list_all_accounts},
    },
    impl_responder,
    site::{auth_guard::CookieAuthenticatedUser, shared::base_template},
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
        (base_template())
        body class="bg-gray-50 min-h-screen py-12 px-4 sm:px-6 lg:px-8" {
            div class="max-w-4xl mx-auto" {
                div class="text-center mb-8" {
                    h1 class="text-3xl font-bold text-gray-900 mb-2" { "Your Accounts" }
                    p class="text-sm text-gray-600" { "Manage your investment accounts" }
                }

                div class="bg-white rounded-lg shadow-md p-6" {
                    div class="flex justify-between items-center mb-6" {
                        h2 class="text-xl font-semibold text-gray-900" { "Account List" }
                        a href="/accounts/create" 
                          class="inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition duration-200" {
                            "Add Account"
                        }
                    }

                    @if accounts.is_empty() {
                        div class="text-center py-12" {
                            div class="text-gray-400 mb-4" {
                                // Simple icon representation using text
                                p class="text-6xl mb-4" { "🏦" }
                            }
                            h3 class="text-lg font-medium text-gray-900 mb-2" { "No accounts yet" }
                            p class="text-gray-500 mb-6" { "Create your first investment account to get started" }
                            a href="/accounts/create" 
                              class="inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition duration-200" {
                                "Create Your First Account"
                            }
                        }
                    } @else {
                        // Calculate summary values before iterating
                        @let total_accounts = accounts.len();
                        
                        div class="overflow-x-auto" {
                            table class="min-w-full divide-y divide-gray-200" {
                                thead class="bg-gray-50" {
                                    tr {
                                        th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider" { "Account Name" }
                                        th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider" { "Account ID" }
                                        th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider" { "Created" }
                                        th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider" { "Actions" }
                                    }
                                }
                                tbody class="bg-white divide-y divide-gray-200" {
                                    @for account in &accounts {
                                        tr class="hover:bg-gray-50 transition duration-200" {
                                            td class="px-6 py-4 whitespace-nowrap" {
                                                div class="flex items-center" {
                                                    div {
                                                        div class="text-sm font-medium text-gray-900" { (account.name) }
                                                        div class="text-sm text-gray-500" { "Investment Account" }
                                                    }
                                                }
                                            }
                                            td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500" {
                                                @if let Some(id) = account.id {
                                                    (format!("#{}", id))
                                                } @else {
                                                    "N/A"
                                                }
                                            }
                                            td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500" {
                                                // Since we don't have created_at field, we'll show a placeholder
                                                "Recent"
                                            }
                                            td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium" {
                                                @if let Some(id) = account.id {
                                                    a href=(format!("/accounts/{}/transactions", id)) 
                                                      class="text-blue-600 hover:text-blue-900 mr-4" {
                                                        "View Transactions"
                                                    }
                                                }
                                                // Edit and delete options can be added later
                                                span class="text-gray-400" { "•••" }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Summary section
                        div class="mt-8 grid grid-cols-1 md:grid-cols-2 gap-4" {
                            div class="bg-gray-50 p-4 rounded-lg" {
                                h4 class="text-sm font-medium text-gray-500" { "Total Accounts" }
                                p class="text-2xl font-bold text-gray-900" { (total_accounts) }
                            }
                            div class="bg-gray-50 p-4 rounded-lg" {
                                h4 class="text-sm font-medium text-gray-500" { "Active Accounts" }
                                p class="text-2xl font-bold text-green-600" { (total_accounts) }
                            }
                        }

                        // Quick navigation
                        div class="mt-8 bg-blue-50 border border-blue-200 rounded-lg p-4" {
                            h4 class="text-sm font-medium text-blue-900 mb-2" { "Quick Actions" }
                            div class="flex flex-wrap gap-2" {
                                a href="/transactions" 
                                  class="inline-flex items-center px-3 py-2 border border-blue-300 rounded-md text-sm font-medium text-blue-700 bg-blue-100 hover:bg-blue-200 transition duration-200" {
                                    "View All Transactions"
                                }
                                a href="/transactions/create" 
                                  class="inline-flex items-center px-3 py-2 border border-blue-300 rounded-md text-sm font-medium text-blue-700 bg-blue-100 hover:bg-blue-200 transition duration-200" {
                                    "Add Transaction"
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}