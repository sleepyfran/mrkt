use crate::api::responses::{ApiResult, bad_request};

/// Validates that the length of a string is within a specified range.
pub fn validate_length(input: &str, min: usize, max: usize) -> ApiResult<()> {
    if input.len() < min || input.len() > max {
        Err(bad_request())
    } else {
        Ok(())
    }
}
