use rocket::Route;

mod login;
mod register;

/// All routes related to accounts.
pub fn routes() -> Vec<Route> {
    routes![register::register, login::login]
}
