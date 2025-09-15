use rocket::Route;

pub mod accounts;
pub mod auth_guard;
pub mod responses;
pub mod transactions;
pub mod users;

/// All routes for the API.
pub fn routes() -> Vec<Route> {
    let mut routes = Vec::new();

    routes.append(&mut accounts::routes());
    routes.append(&mut transactions::routes());
    routes.append(&mut users::routes());

    routes
}
