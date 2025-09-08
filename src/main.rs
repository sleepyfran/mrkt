#[macro_use]
extern crate rocket;

mod api;
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
