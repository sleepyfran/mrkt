use rocket::fs::{FileServer, relative};

#[macro_use]
extern crate rocket;

mod api;
mod core;
mod site;

#[launch]
async fn rocket() -> _ {
    let state = core::init().await;

    rocket::build()
        .mount("/api", api::routes())
        .mount("/", site::routes())
        .mount("/public", FileServer::from(relative!("static")))
        .manage(state)
}
