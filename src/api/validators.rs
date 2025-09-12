/// Validates that the length of a string is within a specified range.
pub fn validate_length(input: &str, min: usize, max: usize) -> Result<(), ()> {
    if input.len() < min || input.len() > max {
        Err(())
    } else {
        Ok(())
    }
}
