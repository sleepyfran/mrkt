pub mod create;
pub mod delete;
pub mod list_all;

/// All routes related to accounts on the site.
pub fn routes() -> Vec<rocket::Route> {
    routes![
        create::create_page,
        create::create_submit,
        delete::delete,
        list_all::list_all
    ]
}
