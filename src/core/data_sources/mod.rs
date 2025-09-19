pub mod alphavantage;
pub mod currencies;
pub mod market_provider;

use std::sync::Arc;

use dotenvy::dotenv;
pub use market_provider::*;

use crate::core::data_sources::alphavantage::provider::AlphaVantageProvider;

/// Attempts to create and return a market data provider based on environment variables.
/// Currently, only Alpha Vantage is supported if the ALPHAVANTAGE_API_KEY environment
/// variable is set. If no supported provider is configured, the function will panic.
pub fn create_market_provider() -> Arc<dyn MarketProvider> {
    dotenv().ok();

    let alpha_vantage_api_key = std::env::var("ALPHAVANTAGE_API_KEY");

    if let Some(api_key) = alpha_vantage_api_key.ok() {
        return Arc::new(AlphaVantageProvider::new(api_key));
    }

    panic!("There should be at least one market provider configured via environment variables.")
}
