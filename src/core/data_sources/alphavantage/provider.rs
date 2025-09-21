use crate::core::{
    data_sources::market_provider::{
        DailyStockPrice, ExchangeRate, MarketDataError, MarketDataResult, MarketProvider,
        StockPriceData, SymbolSearchResult,
    },
    logger,
};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Mutex;
use time::{Date, Duration, OffsetDateTime, PrimitiveDateTime, Time};
use url::Url;

/// Cache entry that holds data with an expiration time.
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    data: T,
    expires_at: OffsetDateTime,
}

impl<T> CacheEntry<T> {
    fn new(data: T, ttl: Duration) -> Self {
        Self {
            data,
            expires_at: OffsetDateTime::now_utc() + ttl,
        }
    }

    fn is_expired(&self) -> bool {
        OffsetDateTime::now_utc() > self.expires_at
    }
}

/// AlphaVantage API provider implementation.
pub struct AlphaVantageProvider {
    api_key: String,
    client: Client,
    base_url: String,
    exchange_rate_cache: Mutex<HashMap<String, CacheEntry<ExchangeRate>>>,
    stock_price_cache: Mutex<HashMap<String, CacheEntry<StockPriceData>>>,
}

impl AlphaVantageProvider {
    /// Create a new AlphaVantage provider with the given API key.
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
            base_url: "https://www.alphavantage.co/query".to_string(),
            exchange_rate_cache: Mutex::new(HashMap::new()),
            stock_price_cache: Mutex::new(HashMap::new()),
        }
    }

    /// Cache TTL is 1 day.
    const CACHE_TTL: Duration = Duration::days(1);

    /// Get exchange rate from cache if available and not expired.
    fn get_cached_exchange_rate(
        &self,
        from_currency: &str,
        to_currency: &str,
    ) -> Option<ExchangeRate> {
        let cache_key = format!("{}-{}", from_currency, to_currency);
        let cache = self.exchange_rate_cache.lock().ok()?;
        let entry = cache.get(&cache_key)?;

        if entry.is_expired() {
            None
        } else {
            Some(entry.data.clone())
        }
    }

    /// Cache an exchange rate.
    fn cache_exchange_rate(&self, from_currency: &str, to_currency: &str, rate: ExchangeRate) {
        let cache_key = format!("{}-{}", from_currency, to_currency);
        if let Ok(mut cache) = self.exchange_rate_cache.lock() {
            cache.insert(cache_key, CacheEntry::new(rate, Self::CACHE_TTL));
        }
    }

    /// Get stock price data from cache if available and not expired.
    fn get_cached_stock_prices(&self, symbol: &str) -> Option<StockPriceData> {
        let cache = self.stock_price_cache.lock().ok()?;
        let entry = cache.get(symbol)?;

        if entry.is_expired() {
            None
        } else {
            Some(entry.data.clone())
        }
    }

    /// Cache stock price data.
    fn cache_stock_prices(&self, symbol: &str, data: StockPriceData) {
        if let Ok(mut cache) = self.stock_price_cache.lock() {
            cache.insert(symbol.to_string(), CacheEntry::new(data, Self::CACHE_TTL));
        }
    }

    /// Build a URL for the AlphaVantage API with the given parameters.
    fn build_url(&self, params: &[(&str, &str)]) -> Result<Url, MarketDataError> {
        let mut url = Url::parse(&self.base_url)
            .map_err(|e| MarketDataError::Network(format!("Invalid base URL: {}", e)))?;

        {
            let mut query_pairs = url.query_pairs_mut();
            for (key, value) in params {
                query_pairs.append_pair(key, value);
            }
            query_pairs.append_pair("apikey", &self.api_key);
        }

        Ok(url)
    }

    /// Parse a date string from AlphaVantage format (YYYY-MM-DD) to OffsetDateTime.
    fn parse_date(date_str: &str) -> Result<OffsetDateTime, MarketDataError> {
        let format = time::format_description::parse("[year]-[month]-[day]")
            .map_err(|e| MarketDataError::Parsing(format!("Invalid format description: {}", e)))?;

        let date = Date::parse(date_str, &format).map_err(|e| {
            MarketDataError::Parsing(format!("Invalid date format '{}': {}", date_str, e))
        })?;

        let time = Time::MIDNIGHT;
        let datetime = PrimitiveDateTime::new(date, time);
        Ok(datetime.assume_utc())
    }

    /// Parse a timestamp string from AlphaVantage format to OffsetDateTime.
    fn parse_timestamp(timestamp_str: &str) -> Result<OffsetDateTime, MarketDataError> {
        // AlphaVantage uses different timestamp formats, handle the most common ones.
        let datetime_format =
            time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]")
                .map_err(|e| {
                    MarketDataError::Parsing(format!("Invalid format description: {}", e))
                })?;

        // Try parsing as a datetime with seconds (without timezone info, assume UTC).
        if let Ok(datetime) = PrimitiveDateTime::parse(timestamp_str, &datetime_format) {
            Ok(datetime.assume_utc())
        } else if let Ok(datetime) = Self::parse_date(timestamp_str) {
            Ok(datetime)
        } else {
            Err(MarketDataError::Parsing(format!(
                "Unable to parse timestamp: {}",
                timestamp_str
            )))
        }
    }
}

// AlphaVantage API response structures.
#[derive(Debug, Deserialize)]
struct AlphaVantageExchangeRateResponse {
    #[serde(rename = "Realtime Currency Exchange Rate")]
    exchange_rate: AlphaVantageExchangeRate,
}

#[derive(Debug, Deserialize)]
struct AlphaVantageExchangeRate {
    #[serde(rename = "1. From_Currency Code")]
    from_currency: String,
    #[serde(rename = "2. From_Currency Name")]
    from_currency_name: String,
    #[serde(rename = "3. To_Currency Code")]
    to_currency: String,
    #[serde(rename = "4. To_Currency Name")]
    to_currency_name: String,
    #[serde(rename = "5. Exchange Rate")]
    exchange_rate: String,
    #[serde(rename = "6. Last Refreshed")]
    last_refreshed: String,
}

#[derive(Debug, Deserialize)]
struct AlphaVantageTimeSeriesResponse {
    #[serde(rename = "Meta Data")]
    meta_data: AlphaVantageMetaData,
    #[serde(rename = "Time Series (Daily)")]
    time_series: HashMap<String, AlphaVantageDailyData>,
}

#[derive(Debug, Deserialize)]
struct AlphaVantageMetaData {
    #[serde(rename = "1. Information")]
    information: String,
    #[serde(rename = "2. Symbol")]
    symbol: String,
    #[serde(rename = "3. Last Refreshed")]
    last_refreshed: String,
    #[serde(rename = "4. Output Size")]
    output_size: String,
    #[serde(rename = "5. Time Zone")]
    time_zone: String,
}

#[derive(Debug, Deserialize)]
struct AlphaVantageDailyData {
    #[serde(rename = "1. open")]
    open: String,
    #[serde(rename = "2. high")]
    high: String,
    #[serde(rename = "3. low")]
    low: String,
    #[serde(rename = "4. close")]
    close: String,
    #[serde(rename = "5. volume")]
    volume: String,
}

#[derive(Debug, Deserialize)]
struct AlphaVantageSearchResponse {
    #[serde(rename = "bestMatches")]
    best_matches: Vec<AlphaVantageSearchMatch>,
}

#[derive(Debug, Deserialize)]
struct AlphaVantageSearchMatch {
    #[serde(rename = "1. symbol")]
    symbol: String,
    #[serde(rename = "2. name")]
    name: String,
    #[serde(rename = "3. type")]
    symbol_type: String,
    #[serde(rename = "4. region")]
    region: String,
    #[serde(rename = "5. marketOpen")]
    market_open: String,
    #[serde(rename = "6. marketClose")]
    market_close: String,
    #[serde(rename = "7. timezone")]
    timezone: String,
    #[serde(rename = "8. currency")]
    currency: String,
    #[serde(rename = "9. matchScore")]
    match_score: String,
}

#[derive(Debug, Deserialize)]
struct AlphaVantageErrorResponse {
    #[serde(rename = "Error Message")]
    error_message: Option<String>,
    #[serde(rename = "Note")]
    note: Option<String>,
}

impl From<reqwest::Error> for MarketDataError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            MarketDataError::Network("Request timeout".to_string())
        } else if error.is_connect() {
            MarketDataError::Network("Connection error".to_string())
        } else {
            MarketDataError::Network(error.to_string())
        }
    }
}

impl From<url::ParseError> for MarketDataError {
    fn from(error: url::ParseError) -> Self {
        MarketDataError::Network(format!("URL parse error: {}", error))
    }
}

impl From<time::error::Parse> for MarketDataError {
    fn from(error: time::error::Parse) -> Self {
        MarketDataError::Parsing(format!("Time parse error: {}", error))
    }
}

impl From<time::error::InvalidFormatDescription> for MarketDataError {
    fn from(error: time::error::InvalidFormatDescription) -> Self {
        MarketDataError::Parsing(format!("Invalid time format description: {}", error))
    }
}

#[async_trait::async_trait]
impl MarketProvider for AlphaVantageProvider {
    async fn get_exchange_rate(
        &self,
        from_currency: &str,
        to_currency: &str,
    ) -> MarketDataResult<ExchangeRate> {
        if let Some(cached_rate) = self.get_cached_exchange_rate(from_currency, to_currency) {
            logger::info!(
                "Using cached exchange rate for {} to {}",
                from_currency,
                to_currency
            );
            return Ok(cached_rate);
        }

        let params = [
            ("function", "CURRENCY_EXCHANGE_RATE"),
            ("from_currency", from_currency),
            ("to_currency", to_currency),
        ];

        let url = self.build_url(&params)?;
        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(MarketDataError::Api(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let response_text = response.text().await?;

        if let Ok(error_response) =
            serde_json::from_str::<AlphaVantageErrorResponse>(&response_text)
        {
            if let Some(error_msg) = error_response.error_message {
                return Err(MarketDataError::Api(error_msg));
            }
            if let Some(note) = error_response.note {
                if note.contains("rate limit") || note.contains("frequency") {
                    return Err(MarketDataError::RateLimit);
                }
                return Err(MarketDataError::Api(note));
            }
        }

        let response_data: AlphaVantageExchangeRateResponse = serde_json::from_str(&response_text)
            .map_err(|e| MarketDataError::Parsing(format!("JSON parse error: {}", e)))?;

        let rate = response_data
            .exchange_rate
            .exchange_rate
            .parse::<f64>()
            .map_err(|e| MarketDataError::Parsing(format!("Invalid exchange rate: {}", e)))?;

        let last_refreshed = Self::parse_timestamp(&response_data.exchange_rate.last_refreshed)?;

        let exchange_rate = ExchangeRate {
            from_currency: response_data.exchange_rate.from_currency,
            to_currency: response_data.exchange_rate.to_currency,
            rate,
            last_refreshed,
        };

        self.cache_exchange_rate(from_currency, to_currency, exchange_rate.clone());

        Ok(exchange_rate)
    }

    async fn get_stock_prices(&self, symbol: &str) -> MarketDataResult<StockPriceData> {
        if let Some(cached_data) = self.get_cached_stock_prices(symbol) {
            logger::info!("Using cached stock prices for {}", symbol);
            return Ok(cached_data);
        }

        let params = [
            ("function", "TIME_SERIES_DAILY"),
            ("symbol", symbol),
            ("outputsize", "compact"), // Get latest 100 data points.
        ];

        let url = self.build_url(&params)?;
        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(MarketDataError::Api(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let response_text = response.text().await?;

        if let Ok(error_response) =
            serde_json::from_str::<AlphaVantageErrorResponse>(&response_text)
        {
            if let Some(error_msg) = error_response.error_message {
                return Err(MarketDataError::InvalidSymbol(error_msg));
            }
            if let Some(note) = error_response.note {
                if note.contains("rate limit") || note.contains("frequency") {
                    return Err(MarketDataError::RateLimit);
                }
                return Err(MarketDataError::Api(note));
            }
        }

        let response_data: AlphaVantageTimeSeriesResponse = serde_json::from_str(&response_text)
            .map_err(|e| MarketDataError::Parsing(format!("JSON parse error: {}", e)))?;

        let mut daily_prices = HashMap::new();

        for (date_str, daily_data) in response_data.time_series {
            let date = Self::parse_date(&date_str)?;

            let open = daily_data
                .open
                .parse::<f64>()
                .map_err(|e| MarketDataError::Parsing(format!("Invalid open price: {}", e)))?;
            let high = daily_data
                .high
                .parse::<f64>()
                .map_err(|e| MarketDataError::Parsing(format!("Invalid high price: {}", e)))?;
            let low = daily_data
                .low
                .parse::<f64>()
                .map_err(|e| MarketDataError::Parsing(format!("Invalid low price: {}", e)))?;
            let close = daily_data
                .close
                .parse::<f64>()
                .map_err(|e| MarketDataError::Parsing(format!("Invalid close price: {}", e)))?;
            let volume = daily_data
                .volume
                .parse::<u64>()
                .map_err(|e| MarketDataError::Parsing(format!("Invalid volume: {}", e)))?;

            daily_prices.insert(
                date_str,
                DailyStockPrice {
                    date,
                    open,
                    close,
                    high,
                    low,
                    volume,
                },
            );
        }

        if daily_prices.is_empty() {
            return Err(MarketDataError::NoData);
        }

        let last_refreshed = Self::parse_timestamp(&response_data.meta_data.last_refreshed)?;

        let stock_price_data = StockPriceData {
            symbol: response_data.meta_data.symbol,
            currency: "EUR".to_string(), // AlphaVantage doesn't always provide currency info.
            daily_prices,
            last_refreshed,
        };

        self.cache_stock_prices(symbol, stock_price_data.clone());

        Ok(stock_price_data)
    }

    async fn search_symbols(&self, keywords: &str) -> MarketDataResult<Vec<SymbolSearchResult>> {
        let params = [("function", "SYMBOL_SEARCH"), ("keywords", keywords)];

        let url = self.build_url(&params)?;
        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(MarketDataError::Api(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let response_text = response.text().await?;

        // Check for error response first
        if let Ok(error_response) =
            serde_json::from_str::<AlphaVantageErrorResponse>(&response_text)
        {
            if let Some(error_msg) = error_response.error_message {
                return Err(MarketDataError::Api(error_msg));
            }
            if let Some(note) = error_response.note {
                if note.contains("rate limit") || note.contains("frequency") {
                    return Err(MarketDataError::RateLimit);
                }
                return Err(MarketDataError::Api(note));
            }
        }

        let response_data: AlphaVantageSearchResponse = serde_json::from_str(&response_text)
            .map_err(|e| MarketDataError::Parsing(format!("JSON parse error: {}", e)))?;

        let mut results = Vec::new();

        for search_match in response_data.best_matches {
            let match_score = search_match
                .match_score
                .parse::<f64>()
                .map_err(|e| MarketDataError::Parsing(format!("Invalid match score: {}", e)))?;

            results.push(SymbolSearchResult {
                symbol: search_match.symbol,
                name: search_match.name,
                currency: search_match.currency,
                match_score,
            });
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = AlphaVantageProvider::new("test_api_key".to_string());
        assert_eq!(provider.api_key, "test_api_key");
        assert_eq!(provider.base_url, "https://www.alphavantage.co/query");
    }

    #[test]
    fn test_cache_entry_creation_and_expiration() {
        use time::Duration;

        let data = "test_data".to_string();
        let entry = CacheEntry::new(data.clone(), Duration::seconds(1));

        assert_eq!(entry.data, data);
        assert!(!entry.is_expired());

        // Test with expired entry
        let expired_entry = CacheEntry::new(data.clone(), Duration::seconds(-1));
        assert!(expired_entry.is_expired());
    }

    #[test]
    fn test_exchange_rate_cache_operations() {
        let provider = AlphaVantageProvider::new("test_key".to_string());

        // Initially, cache should be empty.
        assert!(provider.get_cached_exchange_rate("USD", "EUR").is_none());

        // Create a test exchange rate.
        let exchange_rate = ExchangeRate {
            from_currency: "USD".to_string(),
            to_currency: "EUR".to_string(),
            rate: 0.85,
            last_refreshed: OffsetDateTime::now_utc(),
        };

        // Cache it.
        provider.cache_exchange_rate("USD", "EUR", exchange_rate.clone());

        // Should be able to retrieve it.
        let cached = provider.get_cached_exchange_rate("USD", "EUR");
        assert!(cached.is_some());
        let cached_rate = cached.unwrap();
        assert_eq!(cached_rate.from_currency, "USD");
        assert_eq!(cached_rate.to_currency, "EUR");
        assert_eq!(cached_rate.rate, 0.85);
    }

    #[test]
    fn test_stock_price_cache_operations() {
        let provider = AlphaVantageProvider::new("test_key".to_string());

        // Initially, cache should be empty.
        assert!(provider.get_cached_stock_prices("AAPL").is_none());

        // Create test stock price data.
        let stock_data = StockPriceData {
            symbol: "AAPL".to_string(),
            currency: "USD".to_string(),
            daily_prices: HashMap::new(),
            last_refreshed: OffsetDateTime::now_utc(),
        };

        // Cache it.
        provider.cache_stock_prices("AAPL", stock_data.clone());

        // Should be able to retrieve it.
        let cached = provider.get_cached_stock_prices("AAPL");
        assert!(cached.is_some());
        let cached_data = cached.unwrap();
        assert_eq!(cached_data.symbol, "AAPL");
        assert_eq!(cached_data.currency, "USD");
    }

    #[test]
    fn test_date_parsing() {
        let result = AlphaVantageProvider::parse_date("2023-12-25");
        assert!(result.is_ok());

        let bad_result = AlphaVantageProvider::parse_date("invalid-date");
        assert!(bad_result.is_err());
    }

    #[test]
    fn test_timestamp_parsing() {
        let result = AlphaVantageProvider::parse_timestamp("2023-12-25 15:30:45");
        assert!(result.is_ok());

        let date_result = AlphaVantageProvider::parse_timestamp("2023-12-25");
        assert!(date_result.is_ok());

        let bad_result = AlphaVantageProvider::parse_timestamp("invalid-timestamp");
        assert!(bad_result.is_err());
    }

    #[test]
    fn test_url_building() {
        let provider = AlphaVantageProvider::new("test_key".to_string());
        let params = [("function", "TEST"), ("symbol", "IBM")];
        let url = provider.build_url(&params).unwrap();

        assert!(url.as_str().contains("function=TEST"));
        assert!(url.as_str().contains("symbol=IBM"));
        assert!(url.as_str().contains("apikey=test_key"));
    }
}
