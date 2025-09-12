use time::{Date, macros::format_description};

/// Validates that the length of a string is within a specified range.
pub fn validate_length(input: &str, min: usize, max: usize) -> Result<(), ()> {
    if input.len() < min || input.len() > max {
        Err(())
    } else {
        Ok(())
    }
}

/// Validates that a string is not empty.
pub fn validate_not_empty(input: &str) -> Result<(), ()> {
    if input.is_empty() { Err(()) } else { Ok(()) }
}

/// Validates that a string is a valid date in the format YYYY-MM-DD.
pub fn validate_is_valid_date(input: &str) -> Result<(), ()> {
    Date::parse(input, format_description!("[year]-[month]-[day]"))
        .map(|_| ())
        .map_err(|_| ())
}

/// Validates that a comparable value is greater than a specified value.
pub fn validate_greater_than<T: PartialOrd>(input: T, min: T) -> Result<(), ()> {
    if input <= min { Err(()) } else { Ok(()) }
}
