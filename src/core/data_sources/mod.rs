pub mod currencies;
mod market;
pub mod market_provider;

use std::sync::Arc;

use dotenvy::dotenv;

pub use market::multi_market_provider::MultiMarketProvider;
pub use market_provider::*;

use crate::core::data_sources::market::{
    alphavantage::provider::AlphaVantageProvider, multi_market_provider::Providers,
};

/// Attempts to create and return a market data provider based on environment variables.
/// Currently, only Alpha Vantage is supported if the ALPHAVANTAGE_API_KEY environment
/// variable is set. If no supported provider is configured, the function will panic.
pub fn create_market_provider() -> Arc<dyn MarketProvider> {
    dotenv().ok();

    let mut providers: Providers = Vec::new();

    let alpha_vantage_api_key = std::env::var("ALPHAVANTAGE_API_KEY");
    if let Some(api_key) = alpha_vantage_api_key.ok() {
        providers.push(Arc::new(AlphaVantageProvider::new(api_key)));
    }

    return Arc::new(MultiMarketProvider::new(providers));
}
