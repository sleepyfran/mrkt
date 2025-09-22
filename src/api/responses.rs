use crate::core::data_sources::market_provider::SymbolSearchResult;

/// A standard error response structure that includes an error message.
#[derive(serde::Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// A specialized error response for ticker symbol validation errors.
#[derive(serde::Serialize)]
pub struct TickerSymbolErrorResponse {
    pub error: String,
    pub invalid_symbol: String,
    pub suggestions: Vec<SymbolSearchResult>,
}

/// Macro to implement the `Responder` trait for a custom enum that returns JSON error responses.
/// Uses the Display implementation (from thiserror's #[error] attribute) for error messages.
/// Supports both simple variants and variants with associated data (like #[from] parameters).
/// Example:
/// ```
/// #[derive(Error, Debug)]
/// enum CreateError {
///     #[error("Invalid input")]
///     InvalidPricePerShare,
///     #[error("Database error")]
///     DatabaseError(#[from] DatabaseError),
/// }
///
/// impl_responder! {
///     CreateError {
///         CreateError::InvalidPricePerShare => Status::BadRequest,
///         CreateError::DatabaseError(_) => Status::InternalServerError
///     }
/// }
/// ```
#[macro_export]
macro_rules! impl_responder {
    (
        $error_type:ident {
            $(
                $pattern:pat => $status:expr
            ),* $(,)?
        }
    ) => {
        impl<'r> ::rocket::response::Responder<'r, 'static> for $error_type {
            fn respond_to(self, req: &'r ::rocket::Request<'_>) -> ::rocket::response::Result<'static> {
                let error_response = $crate::api::responses::ErrorResponse {
                    error: self.to_string(),
                };

                let status = match self {
                    $(
                        $pattern => $status,
                    )*
                };

                (status, ::rocket::serde::json::Json(error_response)).respond_to(req)
            }
        }
    };
}
