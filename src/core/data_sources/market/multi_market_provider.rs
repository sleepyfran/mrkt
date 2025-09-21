use std::sync::{Arc, RwLock};

use log::{trace, warn};
use time::Duration;

use crate::core::{
    cache::Cache,
    data_sources::{
        ExchangeRate, MarketDataError, MarketDataResult, MarketProvider, StockPriceData,
        SymbolSearchResult,
    },
};

pub type Providers = Vec<Arc<dyn MarketProvider + Send + Sync>>;

pub struct MultiMarketProvider {
    /// Vector of market data providers that the application supports. Currently
    /// only one provider is included, but already wrapping it in a vector to
    /// facilitate future expansion.
    providers: Providers,

    /// Cache for storing exchange rates with a time-to-live (TTL) to minimize
    /// redundant network requests.
    exchange_rate_cache: RwLock<Cache<String, ExchangeRate>>,

    /// Cache for storing stock prices with a time-to-live (TTL) to minimize
    /// redundant network requests.
    stock_price_cache: RwLock<Cache<String, StockPriceData>>,
}

impl MultiMarketProvider {
    /// Cache TTL for exchange rates, set to 1 week, since currency rates
    /// typically do not fluctuate that much to require more frequent updates
    /// and providers often have strict rate limits.
    const EXCHANGE_RATE_CACHE_TTL: Duration = Duration::weeks(1);

    /// Cache TTL for stock prices, set to 1 day.
    const STOCK_PRICE_CACHE_TTL: Duration = Duration::days(1);

    /// Creates a new instance of `MultiMarketProvider` with the given market
    /// data providers.
    pub fn new(providers: Providers) -> Self {
        if providers.is_empty() {
            panic!("At least one market provider must be provided.");
        }

        Self {
            providers,
            exchange_rate_cache: RwLock::new(Cache::new()),
            stock_price_cache: RwLock::new(Cache::new()),
        }
    }
}

#[async_trait::async_trait]
impl MarketProvider for MultiMarketProvider {
    async fn get_exchange_rate(
        &self,
        from_currency: &str,
        to_currency: &str,
    ) -> MarketDataResult<ExchangeRate> {
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
                    rate = Some(fetched_rate);
                    break;
                }
                Err(e) => {
                    warn!(
                        "Error fetching exchange rate from provider: {}. Trying next provider.",
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
            None => Err(MarketDataError::NoData),
        }
    }

    async fn get_stock_prices(&self, symbol: &str) -> MarketDataResult<StockPriceData> {
        // Attempt to retrieve from cache first.
        {
            let mut cache = self.stock_price_cache.write().expect("Cache lock poisoned");
            if let Some(cached_data) = cache.get_or_delete(&symbol.to_string()) {
                trace!("Using cached stock price data for symbol: {}", symbol);
                return Ok(cached_data.clone());
            }
        }

        // If not in cache, fetch from the first available provider.
        let mut stock_data: Option<StockPriceData> = None;
        for provider in &self.providers {
            match provider.get_stock_prices(symbol).await {
                Ok(fetched_data) => {
                    stock_data = Some(fetched_data);
                    break;
                }
                Err(e) => {
                    warn!(
                        "Error fetching stock prices from provider: {}. Trying next provider.",
                        e
                    );
                }
            }
        }

        match stock_data {
            Some(data) => {
                {
                    let mut cache = self.stock_price_cache.write().expect("Cache lock poisoned");
                    cache.insert(
                        symbol.to_string(),
                        data.clone(),
                        Self::STOCK_PRICE_CACHE_TTL,
                    );
                }
                Ok(data)
            }
            None => Err(MarketDataError::NoData),
        }
    }

    async fn search_symbols(&self, query: &str) -> MarketDataResult<Vec<SymbolSearchResult>> {
        for provider in &self.providers {
            match provider.search_symbols(query).await {
                Ok(results) => return Ok(results),
                Err(e) => {
                    warn!(
                        "Error searching symbols from provider: {}. Trying next provider.",
                        e
                    );
                }
            }
        }
        Err(MarketDataError::NoData)
    }
}
