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
