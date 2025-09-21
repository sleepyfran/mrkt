use crate::core::data_sources::exchange_rate_provider::{
    ExchangeRate, ExchangeRateError, ExchangeRateProvider, ExchangeRateResult,
};
use reqwest::Client;
use serde::Deserialize;
use time::OffsetDateTime;
use url::Url;

/// OpenExchangeRates API provider implementation for exchange rates.
pub struct OpenExchangeRatesProvider {
    app_id: String,
    client: Client,
    base_url: String,
}

impl OpenExchangeRatesProvider {
    /// Create a new OpenExchangeRates provider with the given App ID.
    pub fn new(app_id: String) -> Self {
        Self {
            app_id,
            client: Client::new(),
            base_url: "https://openexchangerates.org/api/".to_string(),
        }
    }

    /// Build a URL for the OpenExchangeRates API with the given parameters.
    fn build_url(&self, endpoint: &str, params: &[(&str, &str)]) -> Result<Url, ExchangeRateError> {
        let mut url = Url::parse(&format!("{}{}", self.base_url, endpoint))
            .map_err(|e| ExchangeRateError::Network(format!("Invalid base URL: {}", e)))?;

        {
            let mut query_pairs = url.query_pairs_mut();
            for (key, value) in params {
                query_pairs.append_pair(key, value);
            }
            query_pairs.append_pair("app_id", &self.app_id);
        }

        Ok(url)
    }
}

// OpenExchangeRates API response structures.
#[derive(Debug, Deserialize)]
struct OpenExchangeRatesResponse {
    disclaimer: String,
    license: String,
    timestamp: i64,
    base: String,
    rates: std::collections::HashMap<String, f64>,
}

#[derive(Debug, Deserialize)]
struct OpenExchangeRatesErrorResponse {
    error: bool,
    status: u16,
    message: String,
    description: String,
}

#[async_trait::async_trait]
impl ExchangeRateProvider for OpenExchangeRatesProvider {
    fn name(&self) -> &'static str {
        "OpenExchangeRates"
    }

    async fn get_exchange_rate(
        &self,
        from_currency: &str,
        to_currency: &str,
    ) -> ExchangeRateResult<ExchangeRate> {
        let from_upper = from_currency.to_uppercase();
        let to_upper = to_currency.to_uppercase();

        // OpenExchangeRates free tier only supports USD as base currency.
        // We need to handle three cases:
        // 1. FROM=USD, TO=other: direct lookup.
        // 2. FROM=other, TO=USD: 1 / rate_from_usd.
        // 3. FROM=other, TO=other: rate_to_usd / rate_from_usd.

        let symbols_param = if from_upper == "USD" && to_upper != "USD" {
            // Case 1: USD to other currency (direct).
            to_upper.clone()
        } else if from_upper != "USD" && to_upper == "USD" {
            // Case 2: Other currency to USD (inverse).
            from_upper.clone()
        } else if from_upper != "USD" && to_upper != "USD" {
            // Case 3: Cross-rate between two non-USD currencies.
            format!("{},{}", from_upper, to_upper)
        } else {
            // Case 4: USD to USD (rate = 1.0).
            return Ok(ExchangeRate {
                from_currency: from_upper,
                to_currency: to_upper,
                rate: 1.0,
                last_refreshed: time::OffsetDateTime::now_utc(),
            });
        };

        let params = [
            ("symbols", symbols_param.as_str()), // Only get the specific currencies we need.
        ];

        let url = self.build_url("latest.json", &params)?;
        let response = self.client.get(url).send().await?;

        let response_text = response.text().await?;

        // Check if this is an error response first.
        if let Ok(error_response) =
            serde_json::from_str::<OpenExchangeRatesErrorResponse>(&response_text)
        {
            if error_response.error {
                return match error_response.status {
                    401 => Err(ExchangeRateError::Api(format!(
                        "Authentication error: {}",
                        error_response.description
                    ))),
                    400 => {
                        if error_response.message.contains("invalid_base") {
                            Err(ExchangeRateError::InvalidCurrency(format!(
                                "Invalid base currency '{}': {}",
                                from_currency, error_response.description
                            )))
                        } else {
                            Err(ExchangeRateError::Api(format!(
                                "Bad request: {}",
                                error_response.description
                            )))
                        }
                    }
                    403 => Err(ExchangeRateError::Api(format!(
                        "Access restricted: {}",
                        error_response.description
                    ))),
                    404 => Err(ExchangeRateError::Api(format!(
                        "Not found: {}",
                        error_response.description
                    ))),
                    429 => Err(ExchangeRateError::RateLimit),
                    _ => Err(ExchangeRateError::Api(format!(
                        "API error {}: {}",
                        error_response.status, error_response.description
                    ))),
                };
            }
        }

        // Try to parse as a successful response.
        let response_data: OpenExchangeRatesResponse = serde_json::from_str(&response_text)
            .map_err(|e| ExchangeRateError::Parsing(format!("JSON parse error: {}", e)))?;

        // The response should always have USD as base for free tier.
        if response_data.base.to_uppercase() != "USD" {
            return Err(ExchangeRateError::Api(format!(
                "Expected base currency 'USD' but got '{}'",
                response_data.base
            )));
        }

        // Calculate the final exchange rate based on the case.
        let final_rate = if from_upper == "USD" && to_upper != "USD" {
            // Case 1: USD to other currency (direct lookup).
            response_data.rates.get(&to_upper).copied().ok_or_else(|| {
                ExchangeRateError::InvalidCurrency(format!(
                    "Currency '{}' not found in response",
                    to_upper
                ))
            })?
        } else if from_upper != "USD" && to_upper == "USD" {
            // Case 2: Other currency to USD (inverse).
            let usd_to_from_rate =
                response_data
                    .rates
                    .get(&from_upper)
                    .copied()
                    .ok_or_else(|| {
                        ExchangeRateError::InvalidCurrency(format!(
                            "Currency '{}' not found in response",
                            from_upper
                        ))
                    })?;

            if usd_to_from_rate == 0.0 {
                return Err(ExchangeRateError::Api(
                    "Cannot calculate inverse rate: rate is zero".to_string(),
                ));
            }

            1.0 / usd_to_from_rate
        } else {
            // Case 3: Cross-rate between two non-USD currencies.
            let usd_to_from_rate =
                response_data
                    .rates
                    .get(&from_upper)
                    .copied()
                    .ok_or_else(|| {
                        ExchangeRateError::InvalidCurrency(format!(
                            "Currency '{}' not found in response",
                            from_upper
                        ))
                    })?;

            let usd_to_to_rate = response_data.rates.get(&to_upper).copied().ok_or_else(|| {
                ExchangeRateError::InvalidCurrency(format!(
                    "Currency '{}' not found in response",
                    to_upper
                ))
            })?;

            if usd_to_from_rate == 0.0 {
                return Err(ExchangeRateError::Api(
                    "Cannot calculate cross rate: from_currency rate is zero".to_string(),
                ));
            }

            usd_to_to_rate / usd_to_from_rate
        };

        // Convert UNIX timestamp to OffsetDateTime.
        let last_refreshed = OffsetDateTime::from_unix_timestamp(response_data.timestamp)
            .map_err(|e| ExchangeRateError::Parsing(format!("Invalid timestamp: {}", e)))?;

        Ok(ExchangeRate {
            from_currency: from_upper,
            to_currency: to_upper,
            rate: final_rate,
            last_refreshed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_name() {
        let provider = OpenExchangeRatesProvider::new("test_app_id".to_string());
        assert_eq!(provider.name(), "OpenExchangeRates");
    }

    #[test]
    fn test_provider_creation() {
        let provider = OpenExchangeRatesProvider::new("test_app_id".to_string());
        assert_eq!(provider.app_id, "test_app_id");
        assert_eq!(provider.base_url, "https://openexchangerates.org/api/");
    }

    #[test]
    fn test_url_building() {
        let provider = OpenExchangeRatesProvider::new("test_app_id".to_string());
        let params = [("symbols", "EUR")];
        let url = provider.build_url("latest.json", &params).unwrap();

        let url_str = url.as_str();
        assert!(url_str.contains("latest.json"));
        assert!(url_str.contains("symbols=EUR"));
        assert!(url_str.contains("app_id=test_app_id"));
        // Note: We don't set 'base' parameter for free tier - it defaults to USD.
    }
}
