pub mod currencies;
pub mod exchange_rate_provider;
mod exchange_rates;
mod market;
pub mod market_provider;

use std::sync::Arc;

use dotenvy::dotenv;

pub use exchange_rate_provider::*;
pub use market::multi_market_provider::MultiMarketProvider;
pub use market_provider::*;

use crate::core::data_sources::exchange_rates::{
    alphavantage::AlphaVantageExchangeRateProvider,
    multi_exchange_rate_provider::ExchangeRateProviders,
    openexchangerates::OpenExchangeRatesProvider,
};
use crate::core::data_sources::market::{
    alphavantage::AlphaVantageProvider, multi_market_provider::Providers,
    yahoo::YahooFinanceProvider,
};
use log::warn;

/// Attempts to create and return a market data provider based on environment variables.
/// Currently, only Alpha Vantage is supported if the ALPHAVANTAGE_API_KEY environment
/// variable is set. If no supported provider is configured, the function will panic.
pub fn create_market_provider() -> Arc<dyn MarketProvider> {
    dotenv().ok();

    let mut providers: Providers = Vec::new();

    let yahoo_connector = YahooFinanceProvider::new();
    match yahoo_connector {
        Ok(connector) => providers.push(Arc::new(connector)),
        Err(e) => warn!(
            "Failed to create Yahoo Finance provider: {}, proceeding without it",
            e
        ),
    }

    let alpha_vantage_api_key = std::env::var("ALPHAVANTAGE_API_KEY");
    if let Some(api_key) = alpha_vantage_api_key.ok() {
        providers.push(Arc::new(AlphaVantageProvider::new(api_key)));
    }

    return Arc::new(MultiMarketProvider::new(providers));
}

/// Attempts to create and return an exchange rate provider based on environment variables.
/// Supports multiple providers: AlphaVantage (ALPHAVANTAGE_API_KEY) and OpenExchangeRates (OPENEXCHANGERATES_APP_ID).
/// If no supported provider is configured, the function will panic.
pub fn create_exchange_rate_provider() -> Arc<dyn ExchangeRateProvider> {
    dotenv().ok();

    let mut providers: ExchangeRateProviders = Vec::new();

    // Try to add OpenExchangeRates provider first (often more reliable for free tier)
    let openexchangerates_app_id = std::env::var("OPENEXCHANGERATES_APP_ID");
    if let Some(app_id) = openexchangerates_app_id.ok() {
        providers.push(Arc::new(OpenExchangeRatesProvider::new(app_id)));
    }

    // Add AlphaVantage as fallback
    let alpha_vantage_api_key = std::env::var("ALPHAVANTAGE_API_KEY");
    if let Some(api_key) = alpha_vantage_api_key.ok() {
        providers.push(Arc::new(AlphaVantageExchangeRateProvider::new(api_key)));
    }

    if providers.is_empty() {
        panic!(
            "No exchange rate providers configured. Please set OPENEXCHANGERATES_APP_ID or ALPHAVANTAGE_API_KEY environment variable."
        );
    }

    return Arc::new(
        exchange_rates::multi_exchange_rate_provider::MultiExchangeRateProvider::new(providers),
    );
}
