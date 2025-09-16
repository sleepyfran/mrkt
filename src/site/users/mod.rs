pub mod login;
pub mod register;

/// All routes for the users section of the site.
pub fn routes() -> Vec<rocket::Route> {
    routes![
        login::login_redirect,
        login::login_page,
        login::login_submit,
        register::register_redirect,
        register::register_page,
        register::register_submit
    ]
}
