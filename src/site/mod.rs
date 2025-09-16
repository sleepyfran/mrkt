mod auth_guard;
mod shared;
mod transactions;
mod users;

/// All routes for the site.
pub fn routes() -> Vec<rocket::Route> {
    let mut routes = Vec::new();

    routes.append(&mut transactions::routes());
    routes.append(&mut users::routes());

    routes
}
