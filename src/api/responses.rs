/// A standard error response structure that includes an error message.
#[derive(serde::Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Macro to implement the `Responder` trait for a custom enum that returns JSON error responses.
/// Uses the Display implementation (from thiserror's #[error] attribute) for error messages.
/// Example:
/// ```
/// #[derive(Error, Debug)]
/// enum CreateError {
///     #[error("Invalid input")]
///     InvalidPricePerShare,
/// }
///
/// impl_responder! {
///     CreateError {
///         InvalidPricePerShare => Status::BadRequest
///     }
/// }
/// ```
#[macro_export]
macro_rules! impl_responder {
    (
        $error_type:ident {
            $(
                $variant:ident => $status:expr
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
                        $error_type::$variant => $status,
                    )*
                };

                (status, ::rocket::serde::json::Json(error_response)).respond_to(req)
            }
        }
    };
}
