use rocket::Route;

mod register;

/// All routes related to accounts.
pub fn routes() -> Vec<Route> {
    routes![register::register]
}
