use std::{collections::HashMap, sync::Arc};
use time::OffsetDateTime;

use crate::core::{
    Account, Transaction, TransactionType, data_sources::{ExchangeRateProvider, MarketProvider}, shared::Amount,
};

// Type aliases for better code readability
type ShareQuantity = Amount;
type TotalCost = Amount;
type Fees = Amount;
type StockData = (ShareQuantity, TotalCost, Fees);

/// Data structure containing processed transaction data, replacing the tuple for better readability.
#[derive(Debug)]
struct ProcessedTransactionData {
    /// Stock positions: ticker -> (shares, total_cost, fees)
    pub stock_positions: HashMap<String, StockData>,
    /// Account values: account_id -> total_value
    pub account_values: HashMap<i64, Amount>,
    /// Total amount invested across all transactions (in EUR)
    pub total_invested: Amount,
    /// Total fees paid across all transactions (in EUR)
    pub total_fees: Amount,
}

#[derive(Debug)]
pub struct PortfolioMetrics {
    pub total_portfolio_value: Amount,
    pub total_invested: Amount,
    pub total_fees: Amount,
    pub net_profit_loss: Amount,
    pub profit_loss_percentage: f64,
    pub total_transactions: usize,
    pub buy_transactions: usize,
    pub sell_transactions: usize,
    pub unique_stocks: usize,
    pub portfolio_breakdown: Vec<StockPosition>,
    pub account_breakdown: Vec<AccountSummary>,
    /// The timestamp when portfolio values were last updated from market data.
    /// None if all values are based on historical cost (no market data available).
    pub last_updated: Option<OffsetDateTime>,
}

#[derive(Debug)]
pub struct StockPosition {
    pub ticker: String,
    pub shares: Amount,
    pub average_cost: Amount,
    pub total_invested: Amount,
    /// Current market value using real-time data from the market provider.
    /// Falls back to average cost if market data is unavailable.
    pub current_value: Amount,
    pub percentage_of_portfolio: f64,
    /// The timestamp when this stock's price was last updated from market data.
    /// None if using historical cost (market data unavailable).
    pub last_updated: Option<OffsetDateTime>,
}

#[derive(Debug)]
pub struct AccountSummary {
    pub account_name: String,
    pub total_value: Amount,
    pub percentage_of_portfolio: f64,
    pub stock_count: usize,
}

impl PortfolioMetrics {
    /// Calculates portfolio metrics from transactions and accounts.
    pub async fn from(
        transactions: &[Transaction],
        accounts: &[Account],
        market_provider: Arc<dyn MarketProvider>,
        exchange_rate_provider: Arc<dyn ExchangeRateProvider>,
    ) -> Self {
        let processed_data =
            Self::process_transactions(transactions, market_provider.as_ref(), exchange_rate_provider.as_ref()).await;

        let transaction_counts = Self::count_transactions(transactions);

        let (portfolio_breakdown, total_current_value, portfolio_last_updated) =
            Self::build_portfolio_breakdown(
                processed_data.stock_positions,
                market_provider.as_ref(),
                exchange_rate_provider.as_ref(),
            )
            .await;

        let account_breakdown = Self::build_account_breakdown(
            accounts,
            processed_data.account_values,
            transactions,
            total_current_value,
        );

        let (net_profit_loss, profit_loss_percentage) =
            Self::calculate_profit_loss(total_current_value, processed_data.total_invested);

        Self {
            total_portfolio_value: total_current_value,
            total_invested: processed_data.total_invested,
            total_fees: processed_data.total_fees,
            net_profit_loss,
            profit_loss_percentage,
            total_transactions: transactions.len(),
            buy_transactions: transaction_counts.0,
            sell_transactions: transaction_counts.1,
            unique_stocks: portfolio_breakdown.len(),
            portfolio_breakdown,
            account_breakdown,
            last_updated: portfolio_last_updated,
        }
    }

    /// Processes all transactions to build stock positions and account values.
    /// TODO: Make the target currency (currently hardcoded to EUR) customizable in the future.
    async fn process_transactions(
        transactions: &[Transaction],
        _market_provider: &dyn MarketProvider,
        exchange_rate_provider: &dyn ExchangeRateProvider,
    ) -> ProcessedTransactionData {
        let mut stock_positions: HashMap<String, StockData> = HashMap::new();
        let mut account_values: HashMap<i64, Amount> = HashMap::new();
        let mut total_invested = 0.0;
        let mut total_fees = 0.0;

        for transaction in transactions {
            let account_id = transaction.account_id;
            let position = stock_positions
                .entry(transaction.ticker_symbol.clone())
                .or_insert((0.0, 0.0, 0.0));

            // Convert transaction values to EUR if needed
            let (eur_price_per_share, eur_fees) = if transaction.currency.to_uppercase() != "EUR" {
                match exchange_rate_provider
                    .get_exchange_rate(&transaction.currency, "EUR")
                    .await
                {
                    Ok(exchange_rate) => (
                        transaction.price_per_share * exchange_rate.rate,
                        transaction.fees * exchange_rate.rate,
                    ),
                    Err(_) => {
                        // Fall back to original values if exchange rate is unavailable
                        (transaction.price_per_share, transaction.fees)
                    }
                }
            } else {
                (transaction.price_per_share, transaction.fees)
            };

            let transaction_value = transaction.share_quantity * eur_price_per_share;

            match transaction.transaction_type {
                TransactionType::Buy => {
                    position.0 += transaction.share_quantity; // shares
                    position.1 += transaction_value; // total_cost
                    position.2 += eur_fees; // fees
                    total_invested += transaction_value;
                    *account_values.entry(account_id).or_insert(0.0) += transaction_value;
                }
                TransactionType::Sell => {
                    position.0 -= transaction.share_quantity; // shares
                    position.1 -= transaction_value; // total_cost
                    position.2 += eur_fees; // fees
                    total_invested -= transaction_value;
                    *account_values.entry(account_id).or_insert(0.0) -= transaction_value;
                }
            }
            total_fees += eur_fees;
        }

        ProcessedTransactionData {
            stock_positions,
            account_values,
            total_invested,
            total_fees,
        }
    }

    /// Counts the number of buy and sell transactions.
    fn count_transactions(transactions: &[Transaction]) -> (usize, usize) {
        let buy_transactions = transactions
            .iter()
            .filter(|t| matches!(t.transaction_type, TransactionType::Buy))
            .count();
        let sell_transactions = transactions
            .iter()
            .filter(|t| matches!(t.transaction_type, TransactionType::Sell))
            .count();

        (buy_transactions, sell_transactions)
    }

    /// Builds portfolio breakdown from stock positions.
    /// TODO: Make the target currency (currently hardcoded to EUR) customizable in the future.
    async fn build_portfolio_breakdown(
        stock_positions: HashMap<String, StockData>,
        market_provider: &dyn MarketProvider,
        exchange_rate_provider: &dyn ExchangeRateProvider,
    ) -> (Vec<StockPosition>, Amount, Option<OffsetDateTime>) {
        let mut portfolio_breakdown = Vec::new();
        let mut total_current_value = 0.0;
        let mut earliest_update: Option<OffsetDateTime> = None;

        for (ticker, (shares, total_cost, _fees)) in &stock_positions {
            if *shares > 0.0 {
                let average_cost = total_cost / shares;
                let mut stock_last_updated: Option<OffsetDateTime> = None;

                // Try to fetch current market price, fall back to average cost if unavailable
                let current_price = match market_provider.get_stock_prices(ticker).await {
                    Ok(stock_data) => {
                        stock_last_updated = Some(stock_data.last_refreshed);

                        // Update earliest_update to track the oldest data in the portfolio
                        match earliest_update {
                            None => earliest_update = Some(stock_data.last_refreshed),
                            Some(current_earliest) => {
                                if stock_data.last_refreshed < current_earliest {
                                    earliest_update = Some(stock_data.last_refreshed);
                                }
                            }
                        }

                        // Get the most recent price from daily prices
                        let base_price = stock_data
                            .daily_prices
                            .values()
                            .max_by_key(|price| price.date)
                            .map(|price| price.close)
                            .unwrap_or(average_cost);

                        // Convert current market price to EUR if the stock is not already in EUR
                        if stock_data.currency.to_uppercase() != "EUR" {
                            match exchange_rate_provider
                                .get_exchange_rate(&stock_data.currency, "EUR")
                                .await
                            {
                                Ok(exchange_rate) => {
                                    // Update earliest_update with exchange rate timestamp if it's older
                                    match earliest_update {
                                        None => {
                                            earliest_update = Some(exchange_rate.last_refreshed)
                                        }
                                        Some(current_earliest) => {
                                            if exchange_rate.last_refreshed < current_earliest {
                                                earliest_update =
                                                    Some(exchange_rate.last_refreshed);
                                            }
                                        }
                                    }
                                    base_price * exchange_rate.rate
                                }
                                Err(_) => {
                                    // Fall back to base price if exchange rate is unavailable
                                    base_price
                                }
                            }
                        } else {
                            base_price
                        }
                    }
                    Err(_) => {
                        // Fall back to average cost if market data is unavailable
                        average_cost
                    }
                };

                let current_value = shares * current_price;
                total_current_value += current_value;

                portfolio_breakdown.push(StockPosition {
                    ticker: ticker.clone(),
                    shares: *shares,
                    average_cost,
                    total_invested: *total_cost,
                    current_value,
                    percentage_of_portfolio: 0.0,
                    last_updated: stock_last_updated,
                });
            }
        }

        // Calculate percentages of portfolio.
        for position in &mut portfolio_breakdown {
            position.percentage_of_portfolio = if total_current_value > 0.0 {
                (position.current_value / total_current_value) * 100.0
            } else {
                0.0
            };
        }

        // Sort by value (largest holdings first).
        portfolio_breakdown.sort_by(|a, b| b.current_value.partial_cmp(&a.current_value).unwrap());

        (portfolio_breakdown, total_current_value, earliest_update)
    }

    /// Builds an account breakdown from account values and transactions.
    fn build_account_breakdown(
        accounts: &[Account],
        account_values: HashMap<i64, Amount>,
        transactions: &[Transaction],
        total_current_value: Amount,
    ) -> Vec<AccountSummary> {
        let mut account_breakdown = Vec::new();

        for account in accounts {
            if let Some(account_id) = account.id {
                let account_value = account_values.get(&account_id).copied().unwrap_or(0.0);
                let stock_count = transactions
                    .iter()
                    .filter(|t| t.account_id == account_id)
                    .map(|t| &t.ticker_symbol)
                    .collect::<std::collections::HashSet<_>>()
                    .len();

                let percentage = if total_current_value > 0.0 {
                    (account_value / total_current_value) * 100.0
                } else {
                    0.0
                };

                account_breakdown.push(AccountSummary {
                    account_name: account.name.clone(),
                    total_value: account_value,
                    percentage_of_portfolio: percentage,
                    stock_count,
                });
            }
        }

        account_breakdown
    }

    /// Calculates profit/loss metrics.
    fn calculate_profit_loss(total_current_value: Amount, total_invested: Amount) -> (Amount, f64) {
        let net_profit_loss = total_current_value - total_invested;
        let profit_loss_percentage = if total_invested > 0.0 {
            (net_profit_loss / total_invested) * 100.0
        } else {
            0.0
        };

        (net_profit_loss, profit_loss_percentage)
    }
}
