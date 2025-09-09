#[macro_use]
extern crate rocket;

mod api;
mod core;
mod db;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .mount("/accounts", api::accounts::routes())
}
