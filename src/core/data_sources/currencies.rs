use std::collections::HashMap;
use std::sync::OnceLock;

/// Represents a physical currency with its code and full name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Currency {
    pub code: String,
    pub name: String,
}

impl Currency {
    pub fn new(code: String, name: String) -> Self {
        Self { code, name }
    }
}

/// Static storage for currencies loaded from the CSV file.
static CURRENCIES: OnceLock<HashMap<String, Currency>> = OnceLock::new();

/// Error type for currency-related operations.
#[derive(Debug, thiserror::Error)]
pub enum CurrencyError {
    #[error("Failed to read currency data file: {0}")]
    FileReadError(#[from] std::io::Error),
    #[error("Failed to parse CSV data: {0}")]
    CsvParseError(String),
    #[error("Currency not found: {0}")]
    CurrencyNotFound(String),
}

/// Loads currencies from the CSV file and returns them as a HashMap.
fn load_currencies_from_csv() -> Result<HashMap<String, Currency>, CurrencyError> {
    // Get the path to the CSV file relative to the binary.
    let csv_content = include_str!("data/physical_currency_list.csv");

    let mut currencies = HashMap::new();
    let mut lines = csv_content.lines();

    // Skip the header line.
    lines.next();

    for (line_number, line) in lines.enumerate() {
        let parts: Vec<&str> = line.split(',').collect();

        if parts.len() != 2 {
            return Err(CurrencyError::CsvParseError(format!(
                "Invalid format at line {}: expected 2 columns, found {}",
                line_number + 2,
                parts.len()
            )));
        }

        let code = parts[0].trim().to_string();
        let name = parts[1].trim().to_string();

        if code.is_empty() || name.is_empty() {
            return Err(CurrencyError::CsvParseError(format!(
                "Empty currency code or name at line {}",
                line_number + 2
            )));
        }

        currencies.insert(code.clone(), Currency::new(code, name));
    }

    Ok(currencies)
}

/// Initializes the currency data by loading it from the CSV file.
fn initialize_currencies() -> &'static HashMap<String, Currency> {
    CURRENCIES.get_or_init(|| {
        load_currencies_from_csv().expect(
            "Failed to load currency data, make sure that the CSV file has been correctly included and not tampered with.",
        )
    })
}

/// Returns a reference to a currency by its code.
pub fn get_currency(code: &str) -> Result<&'static Currency, CurrencyError> {
    let currencies = initialize_currencies();
    currencies
        .get(code.to_uppercase().as_str())
        .ok_or_else(|| CurrencyError::CurrencyNotFound(code.to_string()))
}

/// Returns a reference to all available currencies.
pub fn get_all_currencies() -> &'static HashMap<String, Currency> {
    initialize_currencies()
}

/// Returns a vector of all currency codes sorted alphabetically.
pub fn get_currency_codes() -> Vec<String> {
    let currencies = initialize_currencies();
    let mut codes: Vec<String> = currencies.keys().cloned().collect();
    codes.sort();
    codes
}

/// Checks if a currency code exists in the available currencies.
pub fn is_valid_currency_code(code: &str) -> bool {
    let currencies = initialize_currencies();
    currencies.contains_key(&code.to_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_creation() {
        let currency = Currency::new("EUR".to_string(), "Euro".to_string());
        assert_eq!(currency.code, "EUR");
        assert_eq!(currency.name, "Euro");
    }

    #[test]
    fn test_get_currency() {
        // Test with a known currency.
        let EUR = get_currency("EUR").expect("EUR should exist");
        assert_eq!(EUR.code, "EUR");
        assert_eq!(EUR.name, "Euro");

        // Test case insensitive.
        let EUR_lower = get_currency("EUR").expect("EUR should work");
        assert_eq!(EUR_lower.code, "EUR");

        // Test with non-existent currency.
        assert!(get_currency("XYZ").is_err());
    }

    #[test]
    fn test_is_valid_currency_code() {
        assert!(is_valid_currency_code("EUR"));
        assert!(is_valid_currency_code("EUR")); // Case insensitive.
        assert!(!is_valid_currency_code("XYZ"));
    }

    #[test]
    fn test_get_all_currencies() {
        let currencies = get_all_currencies();
        assert!(!currencies.is_empty());
        assert!(currencies.contains_key("EUR"));
        assert!(currencies.contains_key("EUR"));
        assert!(currencies.contains_key("GBP"));
    }

    #[test]
    fn test_get_currency_codes() {
        let codes = get_currency_codes();
        assert!(!codes.is_empty());
        assert!(codes.contains(&"EUR".to_string()));

        // Verify they're sorted.
        let mut sorted_codes = codes.clone();
        sorted_codes.sort();
        assert_eq!(codes, sorted_codes);
    }
}
