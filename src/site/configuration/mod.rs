mod import;

use rocket::Route;

pub fn routes() -> Vec<Route> {
    let mut routes = routes![
        configuration_page,
    ];
    routes.extend(import::routes());
    routes
}

use maud::{Markup, html, Render};
use rocket::{get, State};

use crate::{
    core::state::CoreState,
    site::{
        auth_guard::CookieAuthenticatedUser,
        shared::{NavSection, Shell},
    },
};

#[get("/configuration")]
pub async fn configuration_page(
    _db_state: &State<CoreState>,
    _auth_user: CookieAuthenticatedUser<'_>,
) -> Markup {
    Shell::create(NavSection::Configuration, "Configuration", html! {
        link rel="stylesheet" href="/static/configuration.css";
        
        page-header {
            page-header-title { "Configuration" }
            page-header-subtitle { "Manage application settings and import data" }
        }

        configuration-sections {
            configuration-section {
                h3 { "Data Import" }
                p { "Import transactions from external portfolio management tools." }
                
                configuration-cards {
                    configuration-card {
                        configuration-card-header {
                            h4 { "Import Transaction Data" }
                            p { "Import transactions from various financial applications like Portfolio Performance (more coming soon)." }
                        }
                        
                        configuration-card-actions {
                            a href="/configuration/import" class="button button-primary" {
                                "Import Transaction Data"
                            }
                        }
                    }
                }
            }
        }
    })
    .add_stylesheet("configuration.css")
    .render()
}