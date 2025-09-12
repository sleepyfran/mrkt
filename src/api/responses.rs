use rocket::{http::Status, response::status};

/// An alias for a custom error type.
pub type ApiError = status::Custom<()>;

/// An alias for a custom empty response type.
pub type EmptyResponse = status::Custom<()>;

/// A type alias for a result with a custom error type.
pub type ApiResult<T> = Result<T, ApiError>;

// ----- 2xx

pub fn created() -> ApiError {
    status::Custom(Status::Created, ())
}

// ----- 4xx

/// Returns a status code of 400 Bad Request.
pub fn bad_request() -> ApiError {
    status::Custom(Status::BadRequest, ())
}

/// Returns a status code of 401 Unauthorized.
pub fn unauthorized() -> ApiError {
    status::Custom(Status::Unauthorized, ())
}

/// Returns a status code of 409 Conflict.
pub fn conflict() -> ApiError {
    status::Custom(Status::Conflict, ())
}

// ----- 5xx

/// Returns a status code of 500 Internal Server Error.
pub fn server_error() -> ApiError {
    status::Custom(Status::InternalServerError, ())
}

/// Macro to implement the `Responder` trait for a custom enum without having to manually bring all
/// all the noise with it. Example:
/// ```
/// impl_responder! {
///     CreateError {
///         CustomError => Status::ImATeapot
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
                match self {
                    $(
                        $error_type::$variant => $status.respond_to(req),
                    )*
                }
            }
        }
    };
}
