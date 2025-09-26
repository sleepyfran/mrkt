use maud::{Markup, html};
use rocket::{
    State,
    form::{Form, FromForm},
    http::{Cookie, CookieJar},
    request::FlashMessage,
    response::{Flash, Redirect},
};

use crate::{
    core::{
        auth::{LoginError, has_any_user, login},
        state::CoreState,
    },
    site::{
        auth_guard::CookieAuthenticatedUser,
        shared::{NavSection, Shell},
    },
};

#[derive(FromForm)]
pub struct LoginForm {
    username: String,
    password: String,
}

/// Redirects authenticated users away from the login page when they are already
/// logged in.
#[get("/login")]
pub async fn login_redirect(_user: CookieAuthenticatedUser<'_>) -> Redirect {
    Redirect::to("/")
}

/// Renders the login page.
#[get("/login", rank = 2)]
pub async fn login_page(
    state: &State<CoreState>,
    flash: Option<FlashMessage<'_>>,
) -> Result<Markup, Redirect> {
    if !has_any_user(&state.pool).await {
        // If there are no users, no point in showing the login page.
        return Err(Redirect::to("/register"));
    }

    Ok(html! {
        (
            Shell::create(NavSection::UserManagement, "Login", html! {
                form-container {
                    page-header {
                        page-header-title { "Sign in to your account" }
                        page-header-subtitle { "Welcome back to mrkt" }
                    }

                    @match flash {
                        Some(flash) => {
                            alert data-alert-type="warning" {
                                p {
                                    strong { "Error: " }
                                    (flash.message())
                                }
                            }
                        }
                        None => { /* No flash message to display */ }
                    }

                    form method="post" action="/login" class="form-card" {
                        form-grid {
                            form-field {
                                label for="username" { "Username" }
                                input
                                    type="text"
                                    id="username"
                                    name="username"
                                    required
                                    placeholder="Enter your username";
                            }

                            form-field {
                                label for="password" { "Password" }
                                input
                                    type="password"
                                    id="password"
                                    name="password"
                                    required
                                    placeholder="Enter your password";
                            }
                        }

                        form-submit {
                            input
                                type="submit"
                                value="Sign in";
                        }
                    }

                    div {
                        p {
                            "Don't have an account? "
                            a href="/register" {
                                "Sign up"
                            }
                        }
                    }
                }
            }).hide_header()
        )
    })
}

/// Handler for login form submission, processes the login and sets a session cookie.
#[post("/login", data = "<form>")]
pub async fn login_submit(
    form: Form<LoginForm>,
    cookies: &CookieJar<'_>,
    state: &State<CoreState>,
) -> Result<Redirect, Flash<Redirect>> {
    match login(&state.pool, &form.username, &form.password).await {
        Ok(token) => {
            let cookie = Cookie::new("session_token", token);
            cookies.add(cookie);

            Ok(Redirect::to("/"))
        }
        Err(LoginError::InvalidCredentials) => Err(Flash::error(
            Redirect::to("/login"),
            "Those credentials are not quite right, try again!",
        )),
        Err(LoginError::DatabaseError(_)) => Err(Flash::error(
            Redirect::to("/login"),
            "Oops, something went wrong with the server or the database. Try again or check the logs for more details",
        )),
    }
}
