use rocket::Route;

mod login;
mod register;
mod session;

/// All routes related to users.
pub fn routes() -> Vec<Route> {
    routes![register::register, login::login]
}
