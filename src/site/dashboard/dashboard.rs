use maud::{Markup, html};
use rocket::State;

use crate::{
    core::{
        Account,
        portfolio::PortfolioMetrics,
        state::CoreState,
        transactions::{ListAllTransactionsError, list_all_transactions},
    },
    site::{
        auth_guard::CookieAuthenticatedUser,
        shared::{NavSection, Shell},
    },
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
        (
            Shell::create(NavSection::Dashboard, "Dashboard", html! {
                div class="" {
                    // Header
                    div class="" {
                        h1 class="" { "Portfolio Dashboard" }
                        p class="" { "Your trading portfolio overview and key metrics" }
                    }

                    @if transactions.is_empty() {
                        // Empty state
                        div class="" {
                            div class="" {
                                p class="" { "📊" }
                            }
                            h3 class="" { "No trading data yet" }
                            p class="" { "Get started by adding your first transaction to see your portfolio metrics" }
                            a href="/transactions/create"
                                class="" {
                                "Add Your First Transaction"
                            }
                        }
                    } @else {
                        // Main dashboard content

                        // Key metrics overview
                        div class="" {
                            // Portfolio Value
                            div class="" {
                                h3 class="" { "Portfolio Value" }
                                p class="" {
                                    (format!("${:.2}", metrics.total_portfolio_value))
                                }
                                p class="" { "Current estimated value" }
                            }

                            // Total Invested
                            div class="" {
                                h3 class="" { "Total Invested" }
                                p class="" {
                                    (format!("${:.2}", metrics.total_invested))
                                }
                                p class="" { "Money put into portfolio" }
                            }

                            // Profit/Loss
                            div class=""
                                    class=(if metrics.net_profit_loss >= 0.0 { "" } else { "" }) {
                                h3 class="" { "Net P&L" }
                                p class=""
                                    class=(if metrics.net_profit_loss >= 0.0 { "" } else { "" }) {
                                    (format!("${:.2}", metrics.net_profit_loss))
                                }
                                p class=""
                                    class=(if metrics.net_profit_loss >= 0.0 { "" } else { "" }) {
                                    (format!("({:.1}%)", metrics.profit_loss_percentage))
                                }
                            }

                            // Total Fees
                            div class="" {
                                h3 class="" { "Total Fees" }
                                p class="" {
                                    (format!("${:.2}", metrics.total_fees))
                                }
                                p class="" { "Transaction costs" }
                            }
                        }

                        // Activity summary
                        div class="" {
                            div class="" {
                                h4 class="" { "Total Transactions" }
                                p class="" { (metrics.total_transactions) }
                            }
                            div class="" {
                                h4 class="" { "Buy Orders" }
                                p class="" { (metrics.buy_transactions) }
                            }
                            div class="" {
                                h4 class="" { "Sell Orders" }
                                p class="" { (metrics.sell_transactions) }
                            }
                            div class="" {
                                h4 class="" { "Unique Stocks" }
                                p class="" { (metrics.unique_stocks) }
                            }
                        }

                        // Portfolio breakdown
                        div class="" {
                            // Holdings breakdown
                            div class="" {
                                h3 class="" { "Current Holdings" }
                                @if metrics.portfolio_breakdown.is_empty() {
                                    p class="" { "No current holdings" }
                                } @else {
                                    div class="" {
                                        @for position in &metrics.portfolio_breakdown {
                                            div class="" {
                                                div class="" {
                                                    div {
                                                        h4 class="" { (position.ticker) }
                                                        p class="" {
                                                            (format!("{:.4} shares @ ${:.2} avg", position.shares, position.average_cost))
                                                        }
                                                    }
                                                    div class="" {
                                                        p class="" {
                                                            (format!("${:.2}", position.current_value))
                                                        }
                                                        p class="" {
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
                            div class="" {
                                h3 class="" { "Account Summary" }
                                @if metrics.account_breakdown.is_empty() {
                                    p class="" { "No accounts found" }
                                } @else {
                                    div class="" {
                                        @for account in &metrics.account_breakdown {
                                            div class="" {
                                                div class="" {
                                                    div {
                                                        h4 class="" { (account.account_name) }
                                                        p class="" {
                                                            (format!("{} different stocks", account.stock_count))
                                                        }
                                                    }
                                                    div class="" {
                                                        p class="" {
                                                            (format!("${:.2}", account.total_value))
                                                        }
                                                        p class="" {
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


                    }
                }
            })
        )
    })
}
