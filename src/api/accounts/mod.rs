use rocket::Route;

mod create;

/// All routes related to accounts.
pub fn routes() -> Vec<Route> {
    routes![create::create]
}
