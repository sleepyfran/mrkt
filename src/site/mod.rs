use rocket::response::Redirect;

use crate::site::auth_guard::CookieAuthenticatedUser;

mod accounts;
mod auth_guard;
mod shared;
mod transactions;
mod users;

#[get("/")]
fn index(_user: CookieAuthenticatedUser) -> Redirect {
    Redirect::to(uri!(transactions::list_all::list_all))
}

fn standard_routes() -> Vec<rocket::Route> {
    routes![index]
}

/// All routes for the site.
pub fn routes() -> Vec<rocket::Route> {
    let mut routes = Vec::new();

    routes.append(&mut standard_routes());
    routes.append(&mut accounts::routes());
    routes.append(&mut transactions::routes());
    routes.append(&mut users::routes());

    routes
}

#[catch(401)]
fn unauthorized_catcher() -> Redirect {
    Redirect::to("/login")
}

/// All catchers for the site.
pub fn catchers() -> Vec<rocket::Catcher> {
    catchers![unauthorized_catcher]
}
