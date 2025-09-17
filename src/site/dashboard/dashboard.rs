use maud::{Markup, html};
use rocket::State;

use crate::{
    core::{
        Account,
        portfolio::PortfolioMetrics,
        state::CoreState,
        transactions::{ListAllTransactionsError, list_all_transactions},
    },
    site::{auth_guard::CookieAuthenticatedUser, shared::base_template},
};

#[get("/dashboard")]
pub async fn dashboard(
    db_state: &State<CoreState>,
    auth_user: CookieAuthenticatedUser<'_>,
) -> Result<Markup, ListAllTransactionsError> {
    let transactions = list_all_transactions(&db_state.pool, auth_user.user_id).await?;
    let accounts = Account::for_user(&db_state.pool, auth_user.user_id)
        .await
        .map_err(|_| ListAllTransactionsError::DatabaseError(sqlx::Error::RowNotFound))?;

    let metrics = PortfolioMetrics::from(&transactions, &accounts);

    Ok(html! {
        (base_template())
        body class="bg-gray-50 min-h-screen py-8 px-4 sm:px-6 lg:px-8" {
            div class="max-w-7xl mx-auto" {
                // Header
                div class="text-center mb-8" {
                    h1 class="text-3xl font-bold text-gray-900 mb-2" { "Portfolio Dashboard" }
                    p class="text-sm text-gray-600" { "Your trading portfolio overview and key metrics" }
                }

                @if transactions.is_empty() {
                    // Empty state
                    div class="bg-white rounded-lg shadow-md p-12 text-center" {
                        div class="text-gray-400 mb-4" {
                            p class="text-6xl mb-4" { "📊" }
                        }
                        h3 class="text-lg font-medium text-gray-900 mb-2" { "No trading data yet" }
                        p class="text-gray-500 mb-6" { "Get started by adding your first transaction to see your portfolio metrics" }
                        a href="/transactions/create"
                          class="inline-flex items-center px-6 py-3 border border-transparent rounded-md shadow-sm text-base font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition duration-200" {
                            "Add Your First Transaction"
                        }
                    }
                } @else {
                    // Main dashboard content

                    // Key metrics overview
                    div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8" {
                        // Portfolio Value
                        div class="bg-white rounded-lg shadow-md p-6 border-l-4 border-blue-500" {
                            h3 class="text-sm font-medium text-gray-500 mb-1" { "Portfolio Value" }
                            p class="text-2xl font-bold text-gray-900" {
                                (format!("${:.2}", metrics.total_portfolio_value))
                            }
                            p class="text-xs text-gray-500 mt-1" { "Current estimated value" }
                        }

                        // Total Invested
                        div class="bg-white rounded-lg shadow-md p-6 border-l-4 border-green-500" {
                            h3 class="text-sm font-medium text-gray-500 mb-1" { "Total Invested" }
                            p class="text-2xl font-bold text-gray-900" {
                                (format!("${:.2}", metrics.total_invested))
                            }
                            p class="text-xs text-gray-500 mt-1" { "Money put into portfolio" }
                        }

                        // Profit/Loss
                        div class="bg-white rounded-lg shadow-md p-6 border-l-4"
                             class=(if metrics.net_profit_loss >= 0.0 { "border-green-500" } else { "border-red-500" }) {
                            h3 class="text-sm font-medium text-gray-500 mb-1" { "Net P&L" }
                            p class="text-2xl font-bold"
                               class=(if metrics.net_profit_loss >= 0.0 { "text-green-600" } else { "text-red-600" }) {
                                (format!("${:.2}", metrics.net_profit_loss))
                            }
                            p class="text-xs"
                               class=(if metrics.net_profit_loss >= 0.0 { "text-green-500" } else { "text-red-500" }) {
                                (format!("({:.1}%)", metrics.profit_loss_percentage))
                            }
                        }

                        // Total Fees
                        div class="bg-white rounded-lg shadow-md p-6 border-l-4 border-yellow-500" {
                            h3 class="text-sm font-medium text-gray-500 mb-1" { "Total Fees" }
                            p class="text-2xl font-bold text-gray-900" {
                                (format!("${:.2}", metrics.total_fees))
                            }
                            p class="text-xs text-gray-500 mt-1" { "Transaction costs" }
                        }
                    }

                    // Activity summary
                    div class="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8" {
                        div class="bg-white rounded-lg shadow-md p-4" {
                            h4 class="text-sm font-medium text-gray-500" { "Total Transactions" }
                            p class="text-xl font-bold text-gray-900" { (metrics.total_transactions) }
                        }
                        div class="bg-white rounded-lg shadow-md p-4" {
                            h4 class="text-sm font-medium text-gray-500" { "Buy Orders" }
                            p class="text-xl font-bold text-green-600" { (metrics.buy_transactions) }
                        }
                        div class="bg-white rounded-lg shadow-md p-4" {
                            h4 class="text-sm font-medium text-gray-500" { "Sell Orders" }
                            p class="text-xl font-bold text-red-600" { (metrics.sell_transactions) }
                        }
                        div class="bg-white rounded-lg shadow-md p-4" {
                            h4 class="text-sm font-medium text-gray-500" { "Unique Stocks" }
                            p class="text-xl font-bold text-blue-600" { (metrics.unique_stocks) }
                        }
                    }

                    // Portfolio breakdown
                    div class="grid grid-cols-1 lg:grid-cols-2 gap-8 mb-8" {
                        // Holdings breakdown
                        div class="bg-white rounded-lg shadow-md p-6" {
                            h3 class="text-lg font-semibold text-gray-900 mb-4" { "Current Holdings" }
                            @if metrics.portfolio_breakdown.is_empty() {
                                p class="text-gray-500 text-center py-8" { "No current holdings" }
                            } @else {
                                div class="space-y-4" {
                                    @for position in &metrics.portfolio_breakdown {
                                        div class="border-l-4 border-blue-500 pl-4 py-2" {
                                            div class="flex justify-between items-start" {
                                                div {
                                                    h4 class="font-semibold text-gray-900" { (position.ticker) }
                                                    p class="text-sm text-gray-600" {
                                                        (format!("{:.4} shares @ ${:.2} avg", position.shares, position.average_cost))
                                                    }
                                                }
                                                div class="text-right" {
                                                    p class="font-semibold text-gray-900" {
                                                        (format!("${:.2}", position.current_value))
                                                    }
                                                    p class="text-sm text-gray-500" {
                                                        (format!("{:.1}%", position.percentage_of_portfolio))
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Account breakdown
                        div class="bg-white rounded-lg shadow-md p-6" {
                            h3 class="text-lg font-semibold text-gray-900 mb-4" { "Account Summary" }
                            @if metrics.account_breakdown.is_empty() {
                                p class="text-gray-500 text-center py-8" { "No accounts found" }
                            } @else {
                                div class="space-y-4" {
                                    @for account in &metrics.account_breakdown {
                                        div class="border-l-4 border-green-500 pl-4 py-2" {
                                            div class="flex justify-between items-start" {
                                                div {
                                                    h4 class="font-semibold text-gray-900" { (account.account_name) }
                                                    p class="text-sm text-gray-600" {
                                                        (format!("{} different stocks", account.stock_count))
                                                    }
                                                }
                                                div class="text-right" {
                                                    p class="font-semibold text-gray-900" {
                                                        (format!("${:.2}", account.total_value))
                                                    }
                                                    p class="text-sm text-gray-500" {
                                                        (format!("{:.1}%", account.percentage_of_portfolio))
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Quick navigation
                    div class="bg-blue-50 border border-blue-200 rounded-lg p-6" {
                        h4 class="text-lg font-medium text-blue-900 mb-4" { "Quick Actions" }
                        div class="grid grid-cols-1 md:grid-cols-4 gap-4" {
                            a href="/transactions/create"
                              class="inline-flex items-center justify-center px-4 py-3 border border-blue-300 rounded-md text-sm font-medium text-blue-700 bg-blue-100 hover:bg-blue-200 transition duration-200" {
                                "Add Transaction"
                            }
                            a href="/transactions"
                              class="inline-flex items-center justify-center px-4 py-3 border border-blue-300 rounded-md text-sm font-medium text-blue-700 bg-blue-100 hover:bg-blue-200 transition duration-200" {
                                "View All Transactions"
                            }
                            a href="/accounts"
                              class="inline-flex items-center justify-center px-4 py-3 border border-blue-300 rounded-md text-sm font-medium text-blue-700 bg-blue-100 hover:bg-blue-200 transition duration-200" {
                                "Manage Accounts"
                            }
                            a href="/accounts/create"
                              class="inline-flex items-center justify-center px-4 py-3 border border-blue-300 rounded-md text-sm font-medium text-blue-700 bg-blue-100 hover:bg-blue-200 transition duration-200" {
                                "Add Account"
                            }
                        }
                    }
                }
            }
        }
    })
}
