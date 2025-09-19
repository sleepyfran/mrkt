use maud::{Markup, html};
use rocket::{
    State,
    form::{Form, FromForm},
    http::{Cookie, CookieJar, Status},
    response::Redirect,
};

use crate::{
    core::{
        auth::{LoginError, login},
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
pub async fn login_page() -> Markup {
    html! {
        (
            Shell::create(NavSection::UserManagement, "Login", html! {
                div class="" {
                    div class="" {
                        h1 class="" { "Sign in to your account" }
                        p class="" { "Welcome back to mrkt" }
                    }

                    form method="post" action="/login" class="" {
                        div class="" {
                            div {
                                label for="username" class="" { "Username" }
                                input
                                    type="text"
                                    id="username"
                                    name="username"
                                    required
                                    class=""
                                    placeholder="Enter your username";
                            }

                            div {
                                label for="password" class="" { "Password" }
                                input
                                    type="password"
                                    id="password"
                                    name="password"
                                    required
                                    class=""
                                    placeholder="Enter your password";
                            }
                        }

                        div class="" {
                            input
                                type="submit"
                                value="Sign in"
                                class="";
                        }
                    }

                    div class="" {
                        p class="" {
                            "Don't have an account? "
                            a href="/register" class="" {
                                "Sign up"
                            }
                        }
                    }
                }
            }).hide_header()
        )
    }
}

/// Handler for login form submission, processes the login and sets a session cookie.
#[post("/login", data = "<form>")]
pub async fn login_submit(
    form: Form<LoginForm>,
    cookies: &CookieJar<'_>,
    state: &State<CoreState>,
) -> Result<Redirect, Status> {
    match login(&state.pool, &form.username, &form.password).await {
        Ok(token) => {
            let cookie = Cookie::new("session_token", token);
            cookies.add(cookie);

            Ok(Redirect::to("/"))
        }
        Err(LoginError::InvalidCredentials) => Ok(Redirect::to("/login")),
        Err(LoginError::DatabaseError(_)) => Err(Status::InternalServerError),
    }
}
