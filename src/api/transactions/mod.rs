use rocket::Route;

mod create;
mod delete;

/// All routes related to transactions.
pub fn routes() -> Vec<Route> {
    routes![create::create, delete::delete]
}
