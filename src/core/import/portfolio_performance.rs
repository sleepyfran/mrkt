use async_trait::async_trait;
use csv::Reader;
use serde::Deserialize;
use std::collections::HashMap;
use std::io::Cursor;

use super::{ImportError, ImportProvider, ImportProviderTrait, ImportResult};
use crate::core::{
    AccountId, TransactionType,
    transactions::{TransactionData, create_transaction},
};

#[derive(Debug, Deserialize)]
struct PortfolioPerformanceRow {
    #[serde(rename = "Date")]
    date: String,
    #[serde(rename = "Type")]
    transaction_type: String,
    #[serde(rename = "Value")]
    value: String,
    #[serde(rename = "Transaction Currency")]
    transaction_currency: String,
    #[serde(rename = "Gross Amount")]
    gross_amount: String,
    #[serde(rename = "Currency Gross Amount")]
    currency_gross_amount: String,
    #[serde(rename = "Exchange Rate")]
    exchange_rate: String,
    #[serde(rename = "Fees")]
    fees: String,
    #[serde(rename = "Taxes")]
    taxes: String,
    #[serde(rename = "Shares")]
    shares: String,
    #[serde(rename = "ISIN")]
    isin: String,
    #[serde(rename = "WKN")]
    wkn: String,
    #[serde(rename = "Ticker Symbol")]
    ticker_symbol: String,
    #[serde(rename = "Security Name")]
    security_name: String,
    #[serde(rename = "Note")]
    note: String,
}

pub struct PortfolioPerformanceProvider;

impl PortfolioPerformanceProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ImportProviderTrait for PortfolioPerformanceProvider {
    async fn import(
        &self,
        pool: &sqlx::SqlitePool,
        market_provider: &dyn crate::core::data_sources::MarketProvider,
        user_id: i64,
        account_id: AccountId,
        file_content: &str,
    ) -> Result<ImportResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut reader = Reader::from_reader(Cursor::new(file_content));
        let mut successful = 0;
        let mut failed = 0;
        let mut errors = Vec::new();
        let mut row_number = 1; // Start from 1 (header is row 0).

        for result in reader.deserialize::<PortfolioPerformanceRow>() {
            row_number += 1;

            let row = match result {
                Ok(row) => row,
                Err(e) => {
                    failed += 1;
                    errors.push(ImportError {
                        row_number,
                        error: format!("CSV parsing error: {}", e),
                        raw_data: HashMap::new(),
                    });
                    continue;
                }
            };

            // Convert Portfolio Performance transaction type to our enum.
            let transaction_type = match row.transaction_type.as_str() {
                "Buy" => TransactionType::Buy,
                "Sell" => TransactionType::Sell,
                "Delivery (Inbound)" => TransactionType::Transfer,
                _ => {
                    failed += 1;
                    let mut raw_data = HashMap::new();
                    raw_data.insert("Type".to_string(), row.transaction_type.clone());
                    errors.push(ImportError {
                        row_number,
                        error: format!("Unsupported transaction type: {}", row.transaction_type),
                        raw_data,
                    });
                    continue;
                }
            };

            // Parse date - Portfolio Performance uses ISO format YYYY-MM-DDTHH:MM.
            let date = if let Some(date_part) = row.date.split('T').next() {
                date_part.to_string()
            } else {
                row.date.clone()
            };

            // Parse numeric fields.
            let share_quantity: f64 = match row.shares.replace(',', ".").parse() {
                Ok(val) => val,
                Err(_) => {
                    failed += 1;
                    let mut raw_data = HashMap::new();
                    raw_data.insert("Shares".to_string(), row.shares.clone());
                    errors.push(ImportError {
                        row_number,
                        error: format!("Invalid share quantity: {}", row.shares),
                        raw_data,
                    });
                    continue;
                }
            };

            let value_str = row.value.replace(",", "");
            let price_per_share = match value_str.parse::<f64>() {
                Ok(amount) => {
                    // Sell orders have negative amounts, but we want positive price per share.
                    let amount = if row.transaction_type == "Sell" {
                        -amount
                    } else {
                        amount
                    };

                    if share_quantity > 0.0 {
                        amount / share_quantity
                    } else {
                        0.0
                    }
                }
                Err(_) => {
                    failed += 1;
                    let mut raw_data = HashMap::new();
                    raw_data.insert("Value".to_string(), value_str.clone());
                    errors.push(ImportError {
                        row_number,
                        error: format!("Invalid transaction value: {}", value_str),
                        raw_data,
                    });
                    continue;
                }
            };

            let fees: f64 = match row.fees.replace(',', ".").parse() {
                Ok(val) => val,
                Err(_) => 0.0, // Default to 0 if fees can't be parsed.
            };

            let transaction_data = TransactionData {
                account_id,
                transaction_type,
                price_per_share,
                share_quantity,
                fees,
                currency: row.transaction_currency.trim().to_string(),
                ticker_symbol: row.ticker_symbol.trim().to_string(),
                date,
            };

            // Attempt to create the transaction.
            match create_transaction(pool, market_provider, user_id, &transaction_data).await {
                Ok(_) => {
                    successful += 1;
                }
                Err(e) => {
                    failed += 1;
                    let mut raw_data = HashMap::new();
                    raw_data.insert("Ticker Symbol".to_string(), row.ticker_symbol.clone());
                    raw_data.insert("Shares".to_string(), row.shares);
                    raw_data.insert("Date".to_string(), row.date);
                    raw_data.insert(
                        "Currency".to_string(),
                        format!("'{}'", row.transaction_currency),
                    );
                    errors.push(ImportError {
                        row_number,
                        error: format!("Transaction creation failed: {}", e),
                        raw_data,
                    });
                }
            }
        }

        Ok(ImportResult {
            successful,
            failed,
            errors,
        })
    }

    fn provider_info(&self) -> ImportProvider {
        ImportProvider {
            name: "Portfolio Performance".to_string(),
            id: "portfolio-performance".to_string(),
            supported_file_types: vec!["csv".to_string()],
            description: "Import transactions from a Portfolio Performance CSV export".to_string(),
        }
    }
}
