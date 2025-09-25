use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ImportResult {
    pub successful: usize,
    pub failed: usize,
    pub errors: Vec<ImportError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportError {
    pub row_number: usize,
    pub error: String,
    pub raw_data: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ImportProvider {
    pub name: String,
    pub id: String,
    pub supported_file_types: Vec<String>,
    pub description: String,
}

#[async_trait]
pub trait ImportProviderTrait: Send + Sync {
    /// Import transactions from the provided file content
    async fn import(
        &self,
        pool: &sqlx::SqlitePool,
        market_provider: &dyn crate::core::data_sources::MarketProvider,
        user_id: i64,
        account_id: crate::core::AccountId,
        file_content: &str,
    ) -> Result<ImportResult, Box<dyn std::error::Error + Send + Sync>>;

    /// Get the metadata for this provider
    fn provider_info(&self) -> ImportProvider;
}

/// Utility function to read file contents from an async reader
pub async fn read_file_contents<R: rocket::tokio::io::AsyncRead + Unpin>(
    mut reader: R,
) -> Result<String, std::io::Error> {
    use rocket::tokio::io::AsyncReadExt;

    let mut contents = String::new();
    reader.read_to_string(&mut contents).await?;
    Ok(contents)
}

pub mod portfolio_performance;

pub use portfolio_performance::PortfolioPerformanceProvider;
