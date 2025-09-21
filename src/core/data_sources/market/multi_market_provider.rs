use std::sync::{Arc, RwLock};

use log::{info, trace, warn};
use time::Duration;

use crate::core::{
    cache::Cache,
    data_sources::{
        MarketDataError, MarketDataResult, MarketProvider, StockPriceData,
        SymbolSearchResult,
    },
};

pub type Providers = Vec<Arc<dyn MarketProvider + Send + Sync>>;

pub struct MultiMarketProvider {
    /// Vector of market data providers that the application supports. Currently
    /// only one provider is included, but already wrapping it in a vector to
    /// facilitate future expansion.
    providers: Providers,

    /// Cache for storing stock prices with a time-to-live (TTL) to minimize
    /// redundant network requests.
    stock_price_cache: RwLock<Cache<String, StockPriceData>>,
}

impl MultiMarketProvider {
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
            stock_price_cache: RwLock::new(Cache::new()),
        }
    }
}

#[async_trait::async_trait]
impl MarketProvider for MultiMarketProvider {
    fn name(&self) -> &'static str {
        "Multi-Provider"
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
                    info!(
                        "Successfully fetched stock prices for {} from {}",
                        symbol,
                        provider.name()
                    );
                    stock_data = Some(fetched_data);
                    break;
                }
                Err(e) => {
                    warn!(
                        "Error fetching stock prices for {} from {}: {}. Trying next provider.",
                        symbol,
                        provider.name(),
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
                Ok(results) => {
                    info!(
                        "Successfully searched symbols for '{}' from {}, found {} results",
                        query,
                        provider.name(),
                        results.len()
                    );
                    return Ok(results);
                }
                Err(e) => {
                    warn!(
                        "Error searching symbols from {}: {}. Trying next provider.",
                        provider.name(),
                        e
                    );
                }
            }
        }
        Err(MarketDataError::NoData)
    }
}
