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
    site::shared::base_template,
};

#[derive(FromForm)]
pub struct RegisterForm {
    username: String,
    password: String,
    confirm_password: String,
}

/// Renders the registration page.
#[get("/register")]
pub async fn register_page(flash: Option<FlashMessage<'_>>) -> Markup {
    html! {
        (base_template())
        body class="bg-gray-50 min-h-screen flex items-center justify-center py-12 px-4 sm:px-6 lg:px-8" {
            div class="max-w-md w-full space-y-8" {
                div class="text-center" {
                    h1 class="text-3xl font-bold text-gray-900 mb-2" { "Create your account" }
                }

                @match flash {
                    Some(flash) => {
                        div class="max-w-md w-full space-y-8" {
                            div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded relative" role="alert" {
                                strong class="font-bold" { "Error: " }
                                span class="block sm:inline" { (flash.message()) }
                            }
                        }
                    }
                    None => { /* No flash message to display */ }
                }

                form method="post" action="/register" class="mt-8 space-y-6 bg-white p-8 rounded-lg shadow-md" {
                    div class="space-y-4" {
                        div {
                            label for="username" class="block text-sm font-medium text-gray-700 mb-1" { "Username" }
                            input
                                type="text"
                                id="username"
                                name="username"
                                required
                                class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition duration-200"
                                placeholder="Choose a username";
                        }

                        div {
                            label for="password" class="block text-sm font-medium text-gray-700 mb-1" { "Password" }
                            input
                                type="password"
                                id="password"
                                name="password"
                                required
                                minlength="8"
                                class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition duration-200"
                                placeholder="Create a password (min 8 characters)";
                        }

                        div {
                            label for="confirm_password" class="block text-sm font-medium text-gray-700 mb-1" { "Confirm Password" }
                            input
                                type="password"
                                id="confirm_password"
                                name="confirm_password"
                                required
                                class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition duration-200"
                                placeholder="Confirm your password";
                        }
                    }

                    div class="pt-4" {
                        input
                            type="submit"
                            value="Create Account"
                            class="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 cursor-pointer transition duration-200";
                    }
                }

                div class="text-center mt-4" {
                    p class="text-sm text-gray-600" {
                        "Already have an account? "
                        a href="/login" class="font-medium text-blue-600 hover:text-blue-500" {
                            "Sign in"
                        }
                    }
                }
            }
        }
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
