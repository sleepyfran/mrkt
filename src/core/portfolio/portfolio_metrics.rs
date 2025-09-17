use std::collections::HashMap;

use crate::core::{Account, Transaction, TransactionType, shared::Amount};

// Type aliases for better code readability
type ShareQuantity = Amount;
type TotalCost = Amount;
type Fees = Amount;
type StockData = (ShareQuantity, TotalCost, Fees);

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
}

#[derive(Debug)]
pub struct StockPosition {
    pub ticker: String,
    pub shares: Amount,
    pub average_cost: Amount,
    pub total_invested: Amount,
    // TODO: Integrate with market data for real current value.
    pub current_value: Amount,
    pub percentage_of_portfolio: f64,
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
    pub fn from(transactions: &[Transaction], accounts: &[Account]) -> Self {
        let (stock_positions, account_values, total_invested, total_fees) =
            Self::process_transactions(transactions);

        let transaction_counts = Self::count_transactions(transactions);

        let (portfolio_breakdown, total_current_value) =
            Self::build_portfolio_breakdown(stock_positions);

        let account_breakdown = Self::build_account_breakdown(
            accounts,
            account_values,
            transactions,
            total_current_value,
        );

        let (net_profit_loss, profit_loss_percentage) =
            Self::calculate_profit_loss(total_current_value, total_invested);

        Self {
            total_portfolio_value: total_current_value,
            total_invested,
            total_fees,
            net_profit_loss,
            profit_loss_percentage,
            total_transactions: transactions.len(),
            buy_transactions: transaction_counts.0,
            sell_transactions: transaction_counts.1,
            unique_stocks: portfolio_breakdown.len(),
            portfolio_breakdown,
            account_breakdown,
        }
    }

    /// Processes all transactions to build stock positions and account values.
    fn process_transactions(
        transactions: &[Transaction],
    ) -> (
        HashMap<String, StockData>,
        HashMap<i64, Amount>,
        Amount,
        Amount,
    ) {
        let mut stock_positions: HashMap<String, StockData> = HashMap::new();
        let mut account_values: HashMap<i64, Amount> = HashMap::new();
        let mut total_invested = 0.0;
        let mut total_fees = 0.0;

        for transaction in transactions {
            let account_id = transaction.account_id;
            let position = stock_positions
                .entry(transaction.ticker_symbol.clone())
                .or_insert((0.0, 0.0, 0.0));
            let transaction_value = transaction.share_quantity * transaction.price_per_share;

            match transaction.transaction_type {
                TransactionType::Buy => {
                    position.0 += transaction.share_quantity; // shares
                    position.1 += transaction_value; // total_cost
                    position.2 += transaction.fees; // fees
                    total_invested += transaction_value;
                    *account_values.entry(account_id).or_insert(0.0) += transaction_value;
                }
                TransactionType::Sell => {
                    position.0 -= transaction.share_quantity; // shares
                    position.1 -= transaction_value; // total_cost
                    position.2 += transaction.fees; // fees
                    total_invested -= transaction_value;
                    *account_values.entry(account_id).or_insert(0.0) -= transaction_value;
                }
            }
            total_fees += transaction.fees;
        }

        (stock_positions, account_values, total_invested, total_fees)
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
    fn build_portfolio_breakdown(
        stock_positions: HashMap<String, StockData>,
    ) -> (Vec<StockPosition>, Amount) {
        let mut portfolio_breakdown = Vec::new();
        let mut total_current_value = 0.0;

        for (ticker, (shares, total_cost, _fees)) in &stock_positions {
            if *shares > 0.0 {
                let average_cost = total_cost / shares;
                // TODO: Fetch market data for this, for now, use average cost as current price.
                let current_value = shares * average_cost;
                total_current_value += current_value;

                portfolio_breakdown.push(StockPosition {
                    ticker: ticker.clone(),
                    shares: *shares,
                    average_cost,
                    total_invested: *total_cost,
                    current_value,
                    percentage_of_portfolio: 0.0,
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

        (portfolio_breakdown, total_current_value)
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
