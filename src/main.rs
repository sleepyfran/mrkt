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
        .mount("/", site::routes())
        .register("/", site::catchers())
        .mount("/api", api::routes())
        .mount("/public", FileServer::from(relative!("static")))
        .manage(state)
}
