pub mod create;
pub mod list;

/// All routes related to transactions on the site.
pub fn routes() -> Vec<rocket::Route> {
    routes![
        create::create_page, 
        create::create_submit, 
        list::list_all,
        list::list_by_account
    ]
}
