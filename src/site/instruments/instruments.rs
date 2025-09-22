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

#[get("/instruments")]
pub async fn instruments(
    db_state: &State<CoreState>,
    auth_user: CookieAuthenticatedUser<'_>,
) -> Result<Markup, ListAllTransactionsError> {
    let transactions = list_all_transactions(&db_state.pool, auth_user.user_id).await?;
    let accounts = Account::for_user(&db_state.pool, auth_user.user_id)
        .await
        .map_err(|_| ListAllTransactionsError::DatabaseError(sqlx::Error::RowNotFound))?;

    let metrics = PortfolioMetrics::from(
        &transactions,
        &accounts,
        db_state.market_provider.clone(),
        db_state.exchange_rate_provider.clone(),
    )
    .await;

    Ok(html! {
        (
            Shell::create(NavSection::Instruments, "Instruments", html! {
                div {
                    page-header {
                        page-header-title { "Your Instruments" }
                        page-header-subtitle { "All stocks in your portfolio with current prices and performance" }
                    }

                    @if metrics.portfolio_breakdown.is_empty() {
                        (empty_state(
                            "📈",
                            "No instruments found",
                            "Start by adding transactions to see your stock instruments",
                            "Add Your First Transaction",
                            "/transactions/create"
                        ))
                    } @else {
                        (instruments_table(&metrics))
                    }
                }
            }).add_stylesheet("instruments.css")
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
                "Just now".to_string()
            } else if diff.whole_minutes() < 60 {
                format!("{} min ago", diff.whole_minutes())
            } else if diff.whole_hours() < 24 {
                format!("{} hrs ago", diff.whole_hours())
            } else {
                format!("{} days ago", diff.whole_days())
            }
        }
        None => "Historical data".to_string(),
    }
}

fn instruments_table(metrics: &PortfolioMetrics) -> Markup {
    html! {
        section-controls {
            h2 { (format!("{} Instruments", metrics.portfolio_breakdown.len())) }
            a href="/transactions/create" {
                "Add Transaction"
            }
        }

        table-container {
            table data-table="content" class="instruments-table" {
                thead {
                    tr {
                        th { "Symbol" }
                        th { "Shares" }
                        th { "Avg. Cost" }
                        th { "Current Price" }
                        th { "Price Change" }
                        th { "Market Value" }
                        th { "P&L" }
                        th { "Last Updated" }
                    }
                }
                tbody {
                    @for position in &metrics.portfolio_breakdown {
                        @let is_positive_price_change = position.price_difference >= 0.0;
                        @let is_positive_pnl = position.total_pnl >= 0.0;

                        tr {
                            td class="instrument-symbol" {
                                div class="symbol-info" {
                                    h4 { (position.ticker) }
                                    p { (format!("{:.1}% of portfolio", position.percentage_of_portfolio)) }
                                }
                            }
                            td class="shares" {
                                (format!("{:.4}", position.shares))
                            }
                            td class="avg-cost" {
                                (format!("{:.2}€", position.average_cost))
                            }
                            td class="current-price" {
                                (format!("{:.2}€", position.current_price))
                            }
                            td class=(if is_positive_price_change { "price-change positive" } else { "price-change negative" }) {
                                div class="price-change-value" {
                                    (format!("{:+.2}€", position.price_difference))
                                }
                                div class="price-change-percent" {
                                    (format!("({:+.1}%)", position.price_change_percentage))
                                }
                            }
                            td class="market-value" {
                                (format!("{:.2}€", position.current_value))
                            }
                            td class=(if is_positive_pnl { "pnl positive" } else { "pnl negative" }) {
                                div class="pnl-value" {
                                    (format!("{:+.2}€", position.total_pnl))
                                }
                                div class="pnl-percent" {
                                    (format!("({:+.1}%)", position.total_pnl_percentage))
                                }
                            }
                            td class="last-updated" {
                                (format_last_update(position.last_updated))
                            }
                        }
                    }
                }
            }
        }

        div class="instruments-summary" {
            p {
                strong { "Total instruments: " } (metrics.portfolio_breakdown.len())
                " • "
                strong { "Portfolio value: " } (format!("{:.2}€", metrics.total_portfolio_value))
                " • "
                strong { "Total P&L: " }
                span class=(if metrics.net_profit_loss >= 0.0 { "positive" } else { "negative" }) {
                    (format!("{:+.2}€ ({:+.1}%)", metrics.net_profit_loss, metrics.profit_loss_percentage))
                }
            }
        }
    }
}
