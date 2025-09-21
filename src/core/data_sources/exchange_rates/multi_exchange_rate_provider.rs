use std::sync::{Arc, RwLock};

use log::{info, trace, warn};
use time::Duration;

use crate::core::{
    cache::Cache,
    data_sources::{
        ExchangeRate, ExchangeRateError, ExchangeRateProvider, ExchangeRateResult,
    },
};

pub type ExchangeRateProviders = Vec<Arc<dyn ExchangeRateProvider + Send + Sync>>;

pub struct MultiExchangeRateProvider {
    /// Vector of exchange rate providers that the application supports.
    providers: ExchangeRateProviders,

    /// Cache for storing exchange rates with a time-to-live (TTL) to minimize
    /// redundant network requests.
    exchange_rate_cache: RwLock<Cache<String, ExchangeRate>>,
}

impl MultiExchangeRateProvider {
    /// Cache TTL for exchange rates, set to 1 week, since currency rates
    /// typically do not fluctuate that much to require more frequent updates
    /// and providers often have strict rate limits.
    const EXCHANGE_RATE_CACHE_TTL: Duration = Duration::weeks(1);

    /// Creates a new instance of `MultiExchangeRateProvider` with the given exchange
    /// rate providers.
    pub fn new(providers: ExchangeRateProviders) -> Self {
        if providers.is_empty() {
            panic!("At least one exchange rate provider must be provided.");
        }

        Self {
            providers,
            exchange_rate_cache: RwLock::new(Cache::new()),
        }
    }
}

#[async_trait::async_trait]
impl ExchangeRateProvider for MultiExchangeRateProvider {
    fn name(&self) -> &'static str {
        "Multi-Exchange-Rate-Provider"
    }

    async fn get_exchange_rate(
        &self,
        from_currency: &str,
        to_currency: &str,
    ) -> ExchangeRateResult<ExchangeRate> {
        let cache_key = format!("{}-{}", from_currency, to_currency);

        // Attempt to retrieve from cache first.
        {
            let mut cache = self
                .exchange_rate_cache
                .write()
                .expect("Cache lock poisoned");
            if let Some(cached_rate) = cache.get_or_delete(&cache_key) {
                trace!(
                    "Using cached exchange rate for {}-{}",
                    from_currency, to_currency
                );
                return Ok(cached_rate.clone());
            }
        }

        // If not in cache, fetch from the first available provider.
        let mut rate: Option<ExchangeRate> = None;
        for provider in &self.providers {
            match provider.get_exchange_rate(from_currency, to_currency).await {
                Ok(fetched_rate) => {
                    info!(
                        "Successfully fetched exchange rate for {}-{} from {}",
                        from_currency,
                        to_currency,
                        provider.name()
                    );
                    rate = Some(fetched_rate);
                    break;
                }
                Err(e) => {
                    warn!(
                        "Error fetching exchange rate from {}: {}. Trying next provider.",
                        provider.name(),
                        e
                    );
                }
            }
        }

        match rate {
            Some(exchange_rate) => {
                {
                    let mut cache = self
                        .exchange_rate_cache
                        .write()
                        .expect("Cache lock poisoned");
                    cache.insert(
                        cache_key,
                        exchange_rate.clone(),
                        Self::EXCHANGE_RATE_CACHE_TTL,
                    );
                }
                Ok(exchange_rate)
            }
            None => Err(ExchangeRateError::NoData),
        }
    }
}