use rocket::Route;

mod create;

/// All routes related to transactions.
pub fn routes() -> Vec<Route> {
    routes![create::create]
}
