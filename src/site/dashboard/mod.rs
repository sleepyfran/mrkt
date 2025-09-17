mod dashboard;

/// All routes for the dashboard.
pub fn routes() -> Vec<rocket::Route> {
    routes![dashboard::dashboard]
}