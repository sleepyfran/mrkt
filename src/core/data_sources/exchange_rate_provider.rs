use time::OffsetDateTime;

/// Represents an exchange rate between two currencies.
#[derive(Debug, Clone)]
pub struct ExchangeRate {
    pub from_currency: String,
    pub to_currency: String,
    pub rate: f64,
    pub last_refreshed: OffsetDateTime,
}

/// Error types for exchange rate operations.
#[derive(Debug, thiserror::Error)]
pub enum ExchangeRateError {
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
    #[error("No data available")]
    NoData,
}

/// Result type for exchange rate operations.
pub type ExchangeRateResult<T> = Result<T, ExchangeRateError>;

impl From<reqwest::Error> for ExchangeRateError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            ExchangeRateError::Network("Request timeout".to_string())
        } else if error.is_connect() {
            ExchangeRateError::Network("Connection error".to_string())
        } else {
            ExchangeRateError::Network(error.to_string())
        }
    }
}

impl From<url::ParseError> for ExchangeRateError {
    fn from(error: url::ParseError) -> Self {
        ExchangeRateError::Network(format!("URL parse error: {}", error))
    }
}

/// Trait defining how to asynchronously retrieve exchange rate data.
#[async_trait::async_trait]
pub trait ExchangeRateProvider: Send + Sync {
    /// Returns the name of this exchange rate provider.
    ///
    /// # Returns
    /// A string identifying this provider (e.g., "AlphaVantage", "ExchangeRatesAPI").
    fn name(&self) -> &'static str;

    /// Retrieves the exchange rate between two currencies.
    ///
    /// # Arguments
    /// * `from_currency` - The source currency code (e.g., "EUR")
    /// * `to_currency` - The target currency code (e.g., "USD")
    ///
    /// # Returns
    /// An `ExchangeRate` containing the rate and last refresh date.
    async fn get_exchange_rate(
        &self,
        from_currency: &str,
        to_currency: &str,
    ) -> ExchangeRateResult<ExchangeRate>;
}
