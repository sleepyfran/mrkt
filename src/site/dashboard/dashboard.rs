use maud::{Markup, html};
use rocket::State;
use time::OffsetDateTime;

use crate::{
    core::{
        Account,
        portfolio::PortfolioMetrics,
        state::CoreState,
        transactions::{ListAllTransactionsError, list_all_transactions},
    },
    site::{
        auth_guard::CookieAuthenticatedUser,
        shared::{NavSection, Shell, empty_state},
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

    let metrics =
        PortfolioMetrics::from(&transactions, &accounts, db_state.market_provider.clone()).await;

    Ok(html! {
        (
            Shell::create(NavSection::Dashboard, "Dashboard", html! {
                div {
                    page-header {
                        page-header-title { "Portfolio Dashboard" }
                        page-header-subtitle { "Your trading portfolio overview and key metrics" }
                    }

                    @if transactions.is_empty() {
                        (empty_state(
                            "📊",
                            "No trading data yet",
                            "Get started by adding your first transaction to see your portfolio metrics",
                            "Add Your First Transaction",
                            "/transactions/create"
                        ))
                    } @else {
                        (key_metrics_overview(&metrics))
                        (activity_summary(&metrics))
                        (portfolio_breakdown(&metrics))
                    }
                }
            }).add_stylesheet("dashboard.css")
        )
    })
}

fn format_last_update(last_updated: Option<OffsetDateTime>) -> String {
    match last_updated {
        Some(timestamp) => {
            // Calculate time difference from now for a more human-readable format
            let now = OffsetDateTime::now_utc();
            let diff = now - timestamp;

            if diff.whole_minutes() < 1 {
                "Last updated: Just now".to_string()
            } else if diff.whole_minutes() < 60 {
                format!("Last updated: {} minutes ago", diff.whole_minutes())
            } else if diff.whole_hours() < 24 {
                format!("Last updated: {} hours ago", diff.whole_hours())
            } else {
                format!("Last updated: {} days ago", diff.whole_days())
            }
        }
        None => "Using historical cost data".to_string(),
    }
}

fn portfolio_value(metrics: &PortfolioMetrics) -> Markup {
    html! {
        div class="" {
            h3 class="" { "Portfolio Value" }
            p class="" {
                (format!("${:.2}", metrics.total_portfolio_value))
            }
            p class="last-updated" { (format_last_update(metrics.last_updated)) }
        }
    }
}

fn total_invested(metrics: &PortfolioMetrics) -> Markup {
    html! {
        div class="" {
            h3 class="" { "Total Invested" }
            p class="" {
                (format!("${:.2}", metrics.total_invested))
            }
            p class="" { "Money put into portfolio" }
        }
    }
}

fn net_profit_loss(metrics: &PortfolioMetrics) -> Markup {
    html! {
        div class=(if metrics.net_profit_loss >= 0.0 { "" } else { "" }) {
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
    }
}

fn total_fees(metrics: &PortfolioMetrics) -> Markup {
    html! {
        div class="" {
            h3 class="" { "Total Fees" }
            p class="" {
                (format!("${:.2}", metrics.total_fees))
            }
            p class="" { "Transaction costs" }
        }
    }
}

fn key_metrics_overview(metrics: &PortfolioMetrics) -> Markup {
    html! {
        div class="key-metrics-overview" {
            (portfolio_value(metrics))
            (total_invested(metrics))
            (net_profit_loss(metrics))
            (total_fees(metrics))
        }
    }
}

fn activity_summary(metrics: &PortfolioMetrics) -> Markup {
    html! {
        div class="activity-summary" {
            div {
                h4 { "Total Transactions" }
                p { (metrics.total_transactions) }
            }
            div {
                h4 { "Buy Orders" }
                p { (metrics.buy_transactions) }
            }
            div {
                h4 { "Sell Orders" }
                p { (metrics.sell_transactions) }
            }
            div {
                h4 { "Unique Stocks" }
                p { (metrics.unique_stocks) }
            }
        }
    }
}

fn current_holdings(metrics: &PortfolioMetrics) -> Markup {
    html! {
        div {
            h3 { "Current Holdings" }
            @if metrics.portfolio_breakdown.is_empty() {
                p { "No current holdings" }
            } @else {
                div class="holdings-list" {
                    @for position in &metrics.portfolio_breakdown {
                        div class="holding-item" {
                            div {
                                div {
                                    h4 { (position.ticker) }
                                    p {
                                        (format!("{:.4} shares @ ${:.2} avg", position.shares, position.average_cost))
                                    }
                                    p class="last-updated" { (format_last_update(position.last_updated)) }
                                }
                                div {
                                    p {
                                        (format!("${:.2}", position.current_value))
                                    }
                                    p {
                                        (format!("{:.1}%", position.percentage_of_portfolio))
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

fn account_summary(metrics: &PortfolioMetrics) -> Markup {
    html! {
        div {
            h3 { "Account Summary" }
            @if metrics.account_breakdown.is_empty() {
                p { "No accounts found" }
            } @else {
                div class="accounts-list" {
                    @for account in &metrics.account_breakdown {
                        div class="account-item" {
                            div {
                                div {
                                    h4 { (account.account_name) }
                                    p {
                                        (format!("{} different stocks", account.stock_count))
                                    }
                                }
                                div {
                                    p {
                                        (format!("${:.2}", account.total_value))
                                    }
                                    p {
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

fn portfolio_breakdown(metrics: &PortfolioMetrics) -> Markup {
    html! {
        div class="portfolio-breakdown" {
            (current_holdings(metrics))
            (account_summary(metrics))
        }
    }
}
