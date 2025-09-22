mod instruments;

/// All routes for the instruments page.
pub fn routes() -> Vec<rocket::Route> {
    routes![instruments::instruments]
}
