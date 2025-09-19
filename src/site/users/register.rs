use maud::{Markup, html};
use rocket::{
    State,
    form::{Form, FromForm},
    get, post,
    request::FlashMessage,
    response::{Flash, Redirect},
};

use crate::{
    core::{
        auth::{RegisterError, register},
        state::CoreState,
    },
    site::{
        auth_guard::CookieAuthenticatedUser,
        shared::{NavSection, Shell},
    },
};

#[derive(FromForm)]
pub struct RegisterForm {
    username: String,
    password: String,
    confirm_password: String,
}

/// Redirects authenticated users away from the registration page when they are
/// already logged in.
#[get("/register")]
pub async fn register_redirect(_user: CookieAuthenticatedUser<'_>) -> Redirect {
    Redirect::to("/")
}

/// Renders the registration page.
#[get("/register", rank = 2)]
pub async fn register_page(flash: Option<FlashMessage<'_>>) -> Markup {
    html! {
        (
            Shell::create(NavSection::UserManagement, "Register", html! {
                form-container {
                    page-header {
                        page-header-title { "Create your account" }
                        page-header-subtitle { "Join mrkt to start tracking your investments" }
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

                    form method="post" action="/register" class="form-card" {
                        form-grid {
                            form-field {
                                label for="username" { "Username" }
                                input
                                    type="text"
                                    id="username"
                                    name="username"
                                    required
                                    placeholder="Choose a username";
                            }

                            form-field {
                                label for="password" { "Password" }
                                input
                                    type="password"
                                    id="password"
                                    name="password"
                                    required
                                    minlength="8"
                                    placeholder="Create a password (min 8 characters)";
                            }

                            form-field {
                                label for="confirm_password" { "Confirm Password" }
                                input
                                    type="password"
                                    id="confirm_password"
                                    name="confirm_password"
                                    required
                                    placeholder="Confirm your password";
                            }
                        }

                        form-submit {
                            input
                                type="submit"
                                value="Create Account";
                        }
                    }

                    div {
                        p {
                            "Already have an account? "
                            a href="/login" {
                                "Sign in"
                            }
                        }
                    }
                }
            }).hide_header()
        )
    }
}

/// Handler for registration form submission, processes the registration.
#[post("/register", data = "<form>")]
pub async fn register_submit(
    form: Form<RegisterForm>,
    state: &State<CoreState>,
) -> Result<Redirect, Flash<Redirect>> {
    // Check if passwords match
    if form.password != form.confirm_password {
        return Err(Flash::error(
            Redirect::to("/register"),
            "Passwords do not match",
        ));
    }

    match register(&state.pool, &form.username, &form.password).await {
        Ok(()) => {
            // Registration successful, redirect to login page
            Ok(Redirect::to("/login"))
        }
        Err(RegisterError::InvalidUsername) => Ok(Redirect::to("/register")),
        Err(RegisterError::InvalidPassword) => Ok(Redirect::to("/register")),
        Err(RegisterError::UsernameAlreadyExists) => Ok(Redirect::to("/register")),
        Err(RegisterError::InternalServerError) => Err(Flash::error(
            Redirect::to("/register"),
            "An internal error occurred. Please try again later.",
        )),
    }
}
