use crate::db::{create_pool, state::DbState};

#[macro_use]
extern crate rocket;

mod api;
mod core;
mod db;

#[launch]
async fn rocket() -> _ {
    // Start by preparing the database.
    let pool = create_pool().await;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to migrate database");

    rocket::build()
        .mount("/accounts", api::accounts::routes())
        .mount("/transactions", api::transactions::routes())
        .mount("/users", api::users::routes())
        .manage(DbState { pool })
}
