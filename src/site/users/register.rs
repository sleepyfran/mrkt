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
        shared::{NavSection, Shell, ShellFlash},
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
                        page-header-title { "Let's get you set up!" }
                        page-header-subtitle { "Create your account and start tracking your investments" }
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
                                    placeholder="Pick a username";
                            }

                            form-field {
                                label for="password" { "Password" }
                                input
                                    type="password"
                                    id="password"
                                    name="password"
                                    required
                                    minlength="8"
                                    placeholder="Make it secure (8+ characters)";
                            }

                            form-field {
                                label for="confirm_password" { "Confirm Password" }
                                input
                                    type="password"
                                    id="confirm_password"
                                    name="confirm_password"
                                    required
                                    placeholder="Same password again";
                            }
                        }

                        form-submit {
                            input
                                type="submit"
                                value="Get started!";
                        }
                    }

                    div {
                        p {
                            "Already set up? "
                            a href="/login" {
                                "Log in"
                            }
                        }
                    }
                }
            })
            .hide_header()
            .attach_flash(flash)
        )
    }
}

/// Handler for registration form submission, processes the registration.
#[post("/register", data = "<form>")]
pub async fn register_submit(
    form: Form<RegisterForm>,
    state: &State<CoreState>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    // Check if passwords match
    if form.password != form.confirm_password {
        return Err(Flash::error(
            Redirect::to("/register"),
            ShellFlash::to_flash_message("Passwords do not match"),
        ));
    }

    match register(&state.pool, &form.username, &form.password).await {
        Ok(()) => Ok(Flash::success(
            Redirect::to("/login"),
            ShellFlash::to_flash_message("Account created successfully! Please log in."),
        )),
        Err(RegisterError::InvalidUsername) => Err(Flash::error(
            Redirect::to("/register"),
            ShellFlash::to_flash_message("Invalid username. Please choose a different username."),
        )),
        Err(RegisterError::InvalidPassword) => Err(Flash::error(
            Redirect::to("/register"),
            ShellFlash::to_flash_message(
                "Invalid password. Password must be at least 8 characters long.",
            ),
        )),
        Err(RegisterError::UsernameAlreadyExists) => Err(Flash::error(
            Redirect::to("/register"),
            ShellFlash::to_flash_message(
                "Username already exists. Please choose a different username.",
            ),
        )),
        Err(RegisterError::InternalServerError) => Err(Flash::error(
            Redirect::to("/register"),
            ShellFlash::to_flash_message("An internal error occurred. Please try again later."),
        )),
    }
}
