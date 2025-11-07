use maud::{Markup, html};
use rocket::State;
use time::OffsetDateTime;

use crate::{
    core::{
        Account,
        data_sources::StockPriceData,
        portfolio::{PortfolioMetrics, StockPosition},
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

    let today = OffsetDateTime::now_utc().date();
    let metrics = PortfolioMetrics::from(
        &transactions,
        &accounts,
        db_state.market_provider.clone(),
        db_state.exchange_rate_provider.clone(),
        today,
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

#[get("/instruments/<symbol>")]
pub async fn instrument_detail(
    symbol: &str,
    db_state: &State<CoreState>,
    auth_user: CookieAuthenticatedUser<'_>,
) -> Result<Markup, ListAllTransactionsError> {
    let transactions = list_all_transactions(&db_state.pool, auth_user.user_id).await?;
    let accounts = Account::for_user(&db_state.pool, auth_user.user_id)
        .await
        .map_err(|_| ListAllTransactionsError::DatabaseError(sqlx::Error::RowNotFound))?;

    let today = OffsetDateTime::now_utc().date();
    let metrics = PortfolioMetrics::from(
        &transactions,
        &accounts,
        db_state.market_provider.clone(),
        db_state.exchange_rate_provider.clone(),
        today,
    )
    .await;

    if let Some(position) = metrics
        .portfolio_breakdown
        .iter()
        .find(|p| p.ticker == symbol)
    {
        // Try to get additional market data for historical prices.
        let historical_data = db_state.market_provider.get_stock_prices(symbol).await.ok();

        Ok(html! {
            (
                Shell::create(NavSection::Instruments, &format!("{} Details", symbol), html! {
                    div {
                        page-header {
                            nav class="breadcrumb" {
                                a href="/instruments" { "← Back to Instruments" }
                            }
                            page-header-title {
                                h1 { (symbol) }
                                span class="instrument-current-price" {
                                    (format!("€{:.2}", position.current_price))
                                }
                            }
                            page-header-subtitle {
                                span class=(if position.price_difference >= 0.0 { "positive" } else { "negative" }) {
                                    (format!("{:+.2}€ ({:+.1}%)", position.price_difference, position.price_change_percentage))
                                }
                            }
                        }

                        (instrument_detail_content(position, &historical_data))
                    }
                }).add_stylesheet("instruments.css")
            )
        })
    } else {
        // Instrument not found in portfolio.
        Ok(html! {
            (
                Shell::create(NavSection::Instruments, "Instrument Not Found", html! {
                    div {
                        page-header {
                            nav class="breadcrumb" {
                                a href="/instruments" { "← Back to Instruments" }
                            }
                            page-header-title { "Instrument Not Found" }
                            page-header-subtitle { "This instrument is not in your current portfolio" }
                        }

                        (empty_state(
                            "❓",
                            &format!("{} not found", symbol),
                            "This instrument is not currently in your portfolio. Add some transactions to track it.",
                            "Add Transaction",
                            "/transactions/create"
                        ))
                    }
                }).add_stylesheet("instruments.css")
            )
        })
    }
}

fn format_last_update(last_updated: Option<OffsetDateTime>) -> String {
    match last_updated {
        Some(timestamp) => {
            // Calculate time difference from now for a more human-readable format.
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

                        tr class="clickable-row" onclick=(format!("window.location.href='/instruments/{}'", position.ticker)) {
                            td class="instrument-symbol" {
                                div class="symbol-info" {
                                    h4 {
                                        a href=(format!("/instruments/{}", position.ticker)) class="symbol-link" {
                                            (position.ticker)
                                        }
                                    }
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
                @if metrics.upcoming_vestings > 0 {
                    " • "
                    strong { "Upcoming vestings: " } (metrics.upcoming_vestings)
                }
            }
        }
    }
}

fn instrument_detail_content(
    position: &StockPosition,
    historical_data: &Option<StockPriceData>,
) -> Markup {
    html! {
        div class="instrument-detail" {
            // Summary cards.
            section class="instrument-summary-cards" {
                div class="summary-card" {
                    h3 { "Position Summary" }
                    div class="card-content" {
                        div class="metric" {
                            span class="metric-label" { "Shares Owned" }
                            span class="metric-value" { (format!("{:.4}", position.shares)) }
                        }
                        div class="metric" {
                            span class="metric-label" { "Average Cost" }
                            span class="metric-value" { (format!("€{:.2}", position.average_cost)) }
                        }
                        div class="metric" {
                            span class="metric-label" { "Total Invested" }
                            span class="metric-value" { (format!("€{:.2}", position.total_invested)) }
                        }
                        div class="metric" {
                            span class="metric-label" { "Market Value" }
                            span class="metric-value" { (format!("€{:.2}", position.current_value)) }
                        }
                    }
                }

                div class="summary-card" {
                    h3 { "Performance" }
                    div class="card-content" {
                        div class="metric" {
                            span class="metric-label" { "Current Price" }
                            span class="metric-value" { (format!("€{:.2}", position.current_price)) }
                        }
                        div class="metric" {
                            span class="metric-label" { "Price Change" }
                            span class=(if position.price_difference >= 0.0 { "metric-value positive" } else { "metric-value negative" }) {
                                (format!("{:+.2}€ ({:+.1}%)", position.price_difference, position.price_change_percentage))
                            }
                        }
                        div class="metric" {
                            span class="metric-label" { "Total P&L" }
                            span class=(if position.total_pnl >= 0.0 { "metric-value positive" } else { "metric-value negative" }) {
                                (format!("{:+.2}€ ({:+.1}%)", position.total_pnl, position.total_pnl_percentage))
                            }
                        }
                        div class="metric" {
                            span class="metric-label" { "Portfolio Weight" }
                            span class="metric-value" { (format!("{:.1}%", position.percentage_of_portfolio)) }
                        }
                    }
                }
            }

            // Historical price data.
            @if let Some(data) = historical_data {
                section class="historical-data" {
                    h3 { "Historical Price Data" }
                    @if data.daily_prices.is_empty() {
                        p class="no-data" { "No historical price data available" }
                    } @else {
                        div class="price-chart" {
                            p class="chart-info" {
                                "Data from " (data.currency) " market • Last updated: " (format_last_update(Some(data.last_refreshed)))
                            }

                            table class="price-history-table" {
                                thead {
                                    tr {
                                        th { "Date" }
                                        th { "Open" }
                                        th { "High" }
                                        th { "Low" }
                                        th { "Close" }
                                        th { "Volume" }
                                    }
                                }
                                tbody {
                                    @for (_, price_data) in data.daily_prices.iter().take(30) {
                                        tr {
                                            td { (price_data.date.format(&time::format_description::well_known::Rfc3339).unwrap_or_default().split('T').next().unwrap_or("")) }
                                            td class=(if price_data.open >= position.average_cost { "price-above-avg" } else { "price-below-avg" }) {
                                                (format!("€{:.2}", price_data.open))
                                            }
                                            td class=(if price_data.high >= position.average_cost { "price-above-avg" } else { "price-below-avg" }) {
                                                (format!("€{:.2}", price_data.high))
                                            }
                                            td class=(if price_data.low >= position.average_cost { "price-above-avg" } else { "price-below-avg" }) {
                                                (format!("€{:.2}", price_data.low))
                                            }
                                            td class=(if price_data.close >= position.average_cost { "price-above-avg" } else { "price-below-avg" }) {
                                                (format!("€{:.2}", price_data.close))
                                            }
                                            td { (price_data.volume.to_string()) }
                                        }
                                    }
                                }
                            }
                            @if data.daily_prices.len() > 30 {
                                p class="data-note" {
                                    "Showing last 30 days of data. Total available: " (data.daily_prices.len()) " days"
                                }
                            }
                        }
                    }
                }
            } @else {
                section class="historical-data" {
                    h3 { "Historical Price Data" }
                    p class="no-data" {
                        "Unable to load historical price data at this time. This could be due to market provider limitations or network issues."
                    }
                }
            }
        }
    }
}
