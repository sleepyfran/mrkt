use crate::core::data_sources::exchange_rate_provider::{
    ExchangeRate, ExchangeRateError, ExchangeRateProvider, ExchangeRateResult,
};
use reqwest::Client;
use serde::Deserialize;
use time::{Date, OffsetDateTime, PrimitiveDateTime, Time};
use url::Url;

/// AlphaVantage API provider implementation for exchange rates.
pub struct AlphaVantageExchangeRateProvider {
    api_key: String,
    client: Client,
    base_url: String,
}

impl AlphaVantageExchangeRateProvider {
    /// Create a new AlphaVantage exchange rate provider with the given API key.
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
            base_url: "https://www.alphavantage.co/query".to_string(),
        }
    }

    /// Build a URL for the AlphaVantage API with the given parameters.
    fn build_url(&self, params: &[(&str, &str)]) -> Result<Url, ExchangeRateError> {
        let mut url = Url::parse(&self.base_url)
            .map_err(|e| ExchangeRateError::Network(format!("Invalid base URL: {}", e)))?;

        {
            let mut query_pairs = url.query_pairs_mut();
            for (key, value) in params {
                query_pairs.append_pair(key, value);
            }
            query_pairs.append_pair("apikey", &self.api_key);
        }

        Ok(url)
    }

    /// Parse a timestamp string from AlphaVantage format to OffsetDateTime.
    fn parse_timestamp(timestamp_str: &str) -> Result<OffsetDateTime, ExchangeRateError> {
        // AlphaVantage uses different timestamp formats, handle the most common ones.
        let datetime_format =
            time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]")
                .map_err(|e| {
                    ExchangeRateError::Parsing(format!("Invalid format description: {}", e))
                })?;

        // Try parsing as a datetime with seconds (without timezone info, assume UTC).
        if let Ok(datetime) = PrimitiveDateTime::parse(timestamp_str, &datetime_format) {
            Ok(datetime.assume_utc())
        } else if let Ok(datetime) = Self::parse_date(timestamp_str) {
            Ok(datetime)
        } else {
            Err(ExchangeRateError::Parsing(format!(
                "Unable to parse timestamp: {}",
                timestamp_str
            )))
        }
    }

    /// Parse a date string from AlphaVantage format (YYYY-MM-DD) to OffsetDateTime.
    fn parse_date(date_str: &str) -> Result<OffsetDateTime, ExchangeRateError> {
        let format = time::format_description::parse("[year]-[month]-[day]").map_err(|e| {
            ExchangeRateError::Parsing(format!("Invalid format description: {}", e))
        })?;

        let date = Date::parse(date_str, &format).map_err(|e| {
            ExchangeRateError::Parsing(format!("Invalid date format '{}': {}", date_str, e))
        })?;

        let time = Time::MIDNIGHT;
        let datetime = PrimitiveDateTime::new(date, time);
        Ok(datetime.assume_utc())
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
struct AlphaVantageErrorResponse {
    #[serde(rename = "Error Message")]
    error_message: Option<String>,
    #[serde(rename = "Note")]
    note: Option<String>,
}

#[async_trait::async_trait]
impl ExchangeRateProvider for AlphaVantageExchangeRateProvider {
    fn name(&self) -> &'static str {
        "AlphaVantage"
    }

    async fn get_exchange_rate(
        &self,
        from_currency: &str,
        to_currency: &str,
    ) -> ExchangeRateResult<ExchangeRate> {
        let params = [
            ("function", "CURRENCY_EXCHANGE_RATE"),
            ("from_currency", from_currency),
            ("to_currency", to_currency),
        ];

        let url = self.build_url(&params)?;
        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(ExchangeRateError::Api(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let response_text = response.text().await?;

        if let Ok(error_response) =
            serde_json::from_str::<AlphaVantageErrorResponse>(&response_text)
        {
            if let Some(error_msg) = error_response.error_message {
                return Err(ExchangeRateError::Api(error_msg));
            }
            if let Some(note) = error_response.note {
                if note.contains("rate limit") || note.contains("frequency") {
                    return Err(ExchangeRateError::RateLimit);
                }
                return Err(ExchangeRateError::Api(note));
            }
        }

        let response_data: AlphaVantageExchangeRateResponse = serde_json::from_str(&response_text)
            .map_err(|e| ExchangeRateError::Parsing(format!("JSON parse error: {}", e)))?;

        let rate = response_data
            .exchange_rate
            .exchange_rate
            .parse::<f64>()
            .map_err(|e| ExchangeRateError::Parsing(format!("Invalid exchange rate: {}", e)))?;

        let last_refreshed = Self::parse_timestamp(&response_data.exchange_rate.last_refreshed)?;

        let exchange_rate = ExchangeRate {
            from_currency: response_data.exchange_rate.from_currency,
            to_currency: response_data.exchange_rate.to_currency,
            rate,
            last_refreshed,
        };

        Ok(exchange_rate)
    }
}
