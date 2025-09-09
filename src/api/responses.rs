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
