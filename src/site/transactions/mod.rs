mod list_all;

/// All routes related to transactions on the site.
pub fn routes() -> Vec<rocket::Route> {
    routes![list_all::list_all]
}
