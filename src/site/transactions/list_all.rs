use maud::{Markup, html};
use rocket::{State, http::Status};

use crate::{
    core::{
        Account, TransactionType,
        state::CoreState,
        transactions::{ListAllTransactionsError, list_all_transactions},
    },
    impl_responder,
    site::{auth_guard::CookieAuthenticatedUser, shared::base_template},
};

impl_responder! {
    ListAllTransactionsError {
        ListAllTransactionsError::DatabaseError(_) => Status::InternalServerError,
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

    Ok(html! {
        (base_template())
        body class="bg-gray-50 min-h-screen py-12 px-4 sm:px-6 lg:px-8" {
            div class="max-w-4xl mx-auto" {
                div class="text-center mb-8" {
                    h1 class="text-3xl font-bold text-gray-900 mb-2" { "Your Transactions" }
                    p class="text-sm text-gray-600" { "View and manage your stock transactions" }
                }

                div class="bg-white rounded-lg shadow-md p-6" {
                    div class="flex justify-between items-center mb-6" {
                        h2 class="text-xl font-semibold text-gray-900" { "Transaction History" }
                        a href="/transactions/create"
                          class="inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition duration-200" {
                            "Add Transaction"
                        }
                    }

                    @if transactions.is_empty() {
                        div class="text-center py-12" {
                            div class="text-gray-400 mb-4" {
                                // Simple icon representation using text
                                p class="text-6xl mb-4" { "📊" }
                            }
                            h3 class="text-lg font-medium text-gray-900 mb-2" { "No transactions yet" }
                            p class="text-gray-500 mb-6" { "Get started by adding your first stock transaction" }
                            a href="/transactions/create"
                              class="inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition duration-200" {
                                "Add Your First Transaction"
                            }
                        }
                    } @else {
                        // Calculate summary values before iterating
                        @let total_transactions = transactions.len();
                        @let buy_count = transactions.iter().filter(|t| matches!(t.transaction_type, TransactionType::Buy)).count();
                        @let sell_count = transactions.iter().filter(|t| matches!(t.transaction_type, TransactionType::Sell)).count();

                        div class="overflow-x-auto" {
                            table class="min-w-full divide-y divide-gray-200" {
                                thead class="bg-gray-50" {
                                    tr {
                                        th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider" { "Date" }
                                        th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider" { "Type" }
                                        th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider" { "Symbol" }
                                        th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider" { "Shares" }
                                        th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider" { "Price" }
                                        th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider" { "Total" }
                                        th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider" { "Account" }
                                        th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider" { "Fees" }
                                    }
                                }
                                tbody class="bg-white divide-y divide-gray-200" {
                                    @for transaction in &transactions {
                                        tr class="hover:bg-gray-50 transition duration-200" {
                                            td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900" {
                                                (transaction.transaction_date)
                                            }
                                            td class="px-6 py-4 whitespace-nowrap" {
                                                @match transaction.transaction_type {
                                                    TransactionType::Buy => {
                                                        span class="inline-flex px-2 py-1 text-xs font-semibold rounded-full bg-green-100 text-green-800" {
                                                            "Buy"
                                                        }
                                                    }
                                                    TransactionType::Sell => {
                                                        span class="inline-flex px-2 py-1 text-xs font-semibold rounded-full bg-red-100 text-red-800" {
                                                            "Sell"
                                                        }
                                                    }
                                                }
                                            }
                                            td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900" {
                                                (transaction.ticker_symbol)
                                            }
                                            td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900" {
                                                (format!("{:.4}", transaction.share_quantity))
                                            }
                                            td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900" {
                                                (format!("{:.2} {}", transaction.price_per_share, transaction.currency))
                                            }
                                            td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900" {
                                                (format!("{:.2} {}",
                                                    transaction.share_quantity * transaction.price_per_share + transaction.fees,
                                                    transaction.currency))
                                            }
                                            td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500" {
                                                @let account_name = accounts.iter()
                                                    .find(|acc| acc.id == Some(transaction.account_id))
                                                    .map(|acc| acc.name.as_str())
                                                    .unwrap_or("Unknown Account");
                                                (account_name)
                                            }
                                            td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500" {
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
                        div class="mt-8 grid grid-cols-1 md:grid-cols-3 gap-4" {
                            div class="bg-gray-50 p-4 rounded-lg" {
                                h4 class="text-sm font-medium text-gray-500" { "Total Transactions" }
                                p class="text-2xl font-bold text-gray-900" { (total_transactions) }
                            }
                            div class="bg-gray-50 p-4 rounded-lg" {
                                h4 class="text-sm font-medium text-gray-500" { "Buy Orders" }
                                p class="text-2xl font-bold text-green-600" { (buy_count) }
                            }
                            div class="bg-gray-50 p-4 rounded-lg" {
                                h4 class="text-sm font-medium text-gray-500" { "Sell Orders" }
                                p class="text-2xl font-bold text-red-600" { (sell_count) }
                            }
                        }

                        /* Quick navigation */
                        div class="mt-8 bg-blue-50 border border-blue-200 rounded-lg p-4" {
                            h4 class="text-sm font-medium text-blue-900 mb-2" { "Quick Actions" }
                            div class="flex flex-wrap gap-2" {
                                a href="/accounts"
                                  class="inline-flex items-center px-3 py-2 border border-blue-300 rounded-md text-sm font-medium text-blue-700 bg-blue-100 hover:bg-blue-200 transition duration-200" {
                                    "Manage Accounts"
                                }
                                a href="/accounts/create"
                                  class="inline-flex items-center px-3 py-2 border border-blue-300 rounded-md text-sm font-medium text-blue-700 bg-blue-100 hover:bg-blue-200 transition duration-200" {
                                    "Add Account"
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}
