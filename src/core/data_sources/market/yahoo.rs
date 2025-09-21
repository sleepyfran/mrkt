/*
Disclaimer: This code was entirely vibe-coded, so here be dragons.
*/

use std::collections::HashMap;
use time::OffsetDateTime;
use yahoo_finance_api as yahoo;

use crate::core::data_sources::market_provider::{
    DailyStockPrice, MarketDataError, MarketDataResult, MarketProvider,
    StockPriceData, SymbolSearchResult,
};

/// Yahoo Finance implementation of the MarketProvider trait.
///
/// This provider uses the Yahoo Finance API to fetch stock prices and search for symbols.
/// Note: Yahoo Finance does not provide direct currency exchange rates, so the
/// get_exchange_rate method will return an error indicating this limitation.
///
/// # Example Usage
///
/// ```rust,no_run
/// use crate::core::data_sources::market::yahoo::YahooFinanceProvider;
/// use crate::core::data_sources::market_provider::MarketProvider;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a new provider
/// let provider = YahooFinanceProvider::new()?;
///
/// // Search for symbols
/// let results = provider.search_symbols("Apple").await?;
/// for result in results.iter().take(3) {
///     println!("{}: {} ({})", result.symbol, result.name, result.currency);
/// }
///
/// // Get stock prices
/// let price_data = provider.get_stock_prices("AAPL").await?;
/// println!("Got {} daily prices for {}",
///          price_data.daily_prices.len(), price_data.symbol);
///
/// // Exchange rates are not supported
/// assert!(provider.get_exchange_rate("USD", "EUR").await.is_err());
/// # Ok(())
/// # }
/// ```
pub struct YahooFinanceProvider {
    connector: yahoo::YahooConnector,
}

impl YahooFinanceProvider {
    /// Creates a new Yahoo Finance provider instance.
    ///
    /// # Returns
    /// A Result containing the provider or an error if initialization fails.
    pub fn new() -> MarketDataResult<Self> {
        let connector = yahoo::YahooConnector::new().map_err(|e| {
            MarketDataError::Api(format!("Failed to create Yahoo connector: {}", e))
        })?;

        Ok(Self { connector })
    }

    /// Converts a Yahoo Finance Quote to our DailyStockPrice format.
    fn convert_quote_to_daily_price(quote: &yahoo::Quote) -> DailyStockPrice {
        DailyStockPrice {
            date: OffsetDateTime::from_unix_timestamp(quote.timestamp)
                .unwrap_or_else(|_| OffsetDateTime::now_utc()),
            open: quote.open,
            close: quote.close,
            high: quote.high,
            low: quote.low,
            volume: quote.volume,
        }
    }

    /// Converts a date to a string key for the HashMap.
    fn date_to_key(date: &OffsetDateTime) -> String {
        date.format(&time::format_description::well_known::Iso8601::DATE)
            .unwrap_or_else(|_| "unknown".to_string())
    }

    /// Determines currency from symbol by looking at exchange suffixes.
    fn infer_currency_from_symbol(symbol: &str) -> String {
        // Common Yahoo Finance symbol suffixes and their currencies
        if symbol.ends_with(".L") || symbol.ends_with(".LON") {
            "GBP".to_string()
        } else if symbol.ends_with(".PA") || symbol.ends_with(".F") || symbol.ends_with(".DE") {
            "EUR".to_string()
        } else if symbol.ends_with(".T") || symbol.ends_with(".TO") {
            "JPY".to_string()
        } else if symbol.ends_with(".HK") {
            "HKD".to_string()
        } else if symbol.ends_with(".AX") {
            "AUD".to_string()
        } else if symbol.ends_with(".TO") {
            "CAD".to_string()
        } else {
            // Default to USD for symbols without clear geographic identifiers
            "USD".to_string()
        }
    }
}

#[async_trait::async_trait]
impl MarketProvider for YahooFinanceProvider {
    fn name(&self) -> &'static str {
        "Yahoo Finance"
    }

    /// Retrieves daily stock price data for a given symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol (e.g., "AAPL", "TSLA")
    ///
    /// # Returns
    /// A `StockPriceData` containing daily OHLC data and last refresh date.
    async fn get_stock_prices(&self, symbol: &str) -> MarketDataResult<StockPriceData> {
        // Fetch historical data for the last 3 months
        let response = self
            .connector
            .get_quote_range(symbol, "1d", "3mo")
            .await
            .map_err(|e| match e {
                yahoo::YahooError::ConnectionFailed(_) => {
                    MarketDataError::Network(format!("Connection failed: {}", e))
                }
                yahoo::YahooError::TooManyRequests(_) => MarketDataError::RateLimit,
                yahoo::YahooError::DeserializeFailed(_) => {
                    MarketDataError::Parsing(format!("Failed to parse response: {}", e))
                }
                yahoo::YahooError::DeserializeFailedDebug(msg) => {
                    MarketDataError::Parsing(format!("Failed to parse response: {}", msg))
                }
                yahoo::YahooError::FetchFailed(_) => {
                    MarketDataError::Api(format!("API fetch error: {}", e))
                }
                _ => MarketDataError::Api(format!("Yahoo Finance API error: {}", e)),
            })?;

        let quotes = response
            .quotes()
            .map_err(|e| MarketDataError::Parsing(format!("Failed to extract quotes: {}", e)))?;

        if quotes.is_empty() {
            return Err(MarketDataError::NoData);
        }

        // Convert quotes to our format
        let mut daily_prices = HashMap::new();
        for quote in &quotes {
            let daily_price = Self::convert_quote_to_daily_price(quote);
            let date_key = Self::date_to_key(&daily_price.date);
            daily_prices.insert(date_key, daily_price);
        }

        // Get the last refreshed time from the most recent quote
        let last_refreshed = quotes
            .last()
            .map(|q| {
                OffsetDateTime::from_unix_timestamp(q.timestamp)
                    .unwrap_or_else(|_| OffsetDateTime::now_utc())
            })
            .unwrap_or_else(OffsetDateTime::now_utc);

        // Infer currency from symbol
        let currency = Self::infer_currency_from_symbol(symbol);

        Ok(StockPriceData {
            symbol: symbol.to_string(),
            currency,
            daily_prices,
            last_refreshed,
        })
    }

    /// Searches for stock symbols based on keywords.
    ///
    /// # Arguments
    /// * `keywords` - Search terms to find matching symbols
    ///
    /// # Returns
    /// A vector of `SymbolSearchResult` with matching symbols and their details.
    async fn search_symbols(&self, keywords: &str) -> MarketDataResult<Vec<SymbolSearchResult>> {
        let response = self
            .connector
            .search_ticker(keywords)
            .await
            .map_err(|e| match e {
                yahoo::YahooError::ConnectionFailed(_) => {
                    MarketDataError::Network(format!("Connection failed: {}", e))
                }
                yahoo::YahooError::TooManyRequests(_) => MarketDataError::RateLimit,
                yahoo::YahooError::DeserializeFailed(_) => {
                    MarketDataError::Parsing(format!("Failed to parse search response: {}", e))
                }
                yahoo::YahooError::DeserializeFailedDebug(msg) => {
                    MarketDataError::Parsing(format!("Failed to parse search response: {}", msg))
                }
                yahoo::YahooError::FetchFailed(_) => {
                    MarketDataError::Api(format!("Search API fetch error: {}", e))
                }
                _ => MarketDataError::Api(format!("Yahoo Finance search error: {}", e)),
            })?;

        let mut results = Vec::new();
        for quote_item in response.quotes {
            // Only include equity symbols (stocks) and filter out empty symbols
            if !quote_item.symbol.is_empty()
                && (quote_item.quote_type == "EQUITY" || quote_item.quote_type == "ETF")
            {
                // Use long_name if available, otherwise fall back to short_name
                let name = if !quote_item.long_name.is_empty() {
                    quote_item.long_name
                } else {
                    quote_item.short_name
                };

                // Infer currency from symbol
                let currency = Self::infer_currency_from_symbol(&quote_item.symbol);

                results.push(SymbolSearchResult {
                    symbol: quote_item.symbol,
                    name,
                    currency,
                    match_score: quote_item.score,
                });
            }
        }

        // Sort by match score (higher is better)
        results.sort_by(|a, b| {
            b.match_score
                .partial_cmp(&a.match_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_inference() {
        assert_eq!(
            YahooFinanceProvider::infer_currency_from_symbol("AAPL"),
            "USD"
        );
        assert_eq!(
            YahooFinanceProvider::infer_currency_from_symbol("TSLA"),
            "USD"
        );
        assert_eq!(
            YahooFinanceProvider::infer_currency_from_symbol("VOD.L"),
            "GBP"
        );
        assert_eq!(
            YahooFinanceProvider::infer_currency_from_symbol("BMW.DE"),
            "EUR"
        );
        assert_eq!(
            YahooFinanceProvider::infer_currency_from_symbol("SONY.T"),
            "JPY"
        );
        assert_eq!(
            YahooFinanceProvider::infer_currency_from_symbol("0700.HK"),
            "HKD"
        );
        assert_eq!(
            YahooFinanceProvider::infer_currency_from_symbol("CBA.AX"),
            "AUD"
        );
    }

    #[test]
    fn test_date_to_key() {
        let date = OffsetDateTime::from_unix_timestamp(1640995200).unwrap(); // 2022-01-01
        let key = YahooFinanceProvider::date_to_key(&date);
        assert_eq!(key, "2022-01-01");
    }

    #[test]
    fn test_provider_name() {
        let provider = YahooFinanceProvider::new().unwrap();
        assert_eq!(provider.name(), "Yahoo Finance");
    }

    #[test]
    fn test_provider_creation() {
        let result = YahooFinanceProvider::new();
        assert!(result.is_ok());
    }

    #[test]
    fn test_yahoo_provider_creation() {
        let result = YahooFinanceProvider::new();
        assert!(result.is_ok());
    }
}
