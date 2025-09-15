pub mod login;

/// All routes for the users section of the site.
pub fn routes() -> Vec<rocket::Route> {
    routes![login::login_page, login::login_submit]
}
