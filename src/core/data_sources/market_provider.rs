use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::OffsetDateTime;

/// Represents daily stock price data.
#[derive(Debug, Clone)]
pub struct DailyStockPrice {
    pub date: OffsetDateTime,
    pub open: f64,
    pub close: f64,
    pub high: f64,
    pub low: f64,
    pub volume: u64,
}

/// Represents a collection of daily stock prices for a symbol.
#[derive(Debug, Clone)]
pub struct StockPriceData {
    pub symbol: String,
    pub currency: String,
    pub daily_prices: HashMap<String, DailyStockPrice>, // Date string as key.
    pub last_refreshed: OffsetDateTime,
}

/// Represents a search result for a stock symbol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolSearchResult {
    pub symbol: String,
    pub name: String,
    pub currency: String,
    pub match_score: f64, // 0.0 to 1.0.
}

/// Error types for market data operations.
#[derive(Debug, thiserror::Error)]
pub enum MarketDataError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("API error: {0}")]
    Api(String),
    #[error("Parsing error: {0}")]
    Parsing(String),
    #[error("Rate limit exceeded")]
    RateLimit,
    #[error("Invalid currency: {0}")]
    InvalidCurrency(String),
    #[error("Invalid symbol: {0}")]
    InvalidSymbol(String),
    #[error("No data available")]
    NoData,
}

/// Result type for market data operations.
pub type MarketDataResult<T> = Result<T, MarketDataError>;

/// Trait defining how to asynchronously retrieve market data.
#[async_trait::async_trait]
pub trait MarketProvider: Send + Sync {
    /// Returns the name of this market data provider.
    ///
    /// # Returns
    /// A string identifying this provider (e.g., "AlphaVantage", "Yahoo Finance").
    fn name(&self) -> &'static str;

    /// Retrieves daily stock price data for a given symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol (e.g., "AAPL")
    ///
    /// # Returns
    /// A `StockPriceData` containing daily OHLC data and last refresh date.
    async fn get_stock_prices(&self, symbol: &str) -> MarketDataResult<StockPriceData>;

    /// Searches for stock symbols based on keywords.
    ///
    /// # Arguments
    /// * `keywords` - Search terms to find matching symbols
    ///
    /// # Returns
    /// A vector of `SymbolSearchResult` with matching symbols and their details.
    async fn search_symbols(&self, keywords: &str) -> MarketDataResult<Vec<SymbolSearchResult>>;
}
