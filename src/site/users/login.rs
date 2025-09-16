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
    site::shared::base_template,
};

#[derive(FromForm)]
pub struct LoginForm {
    username: String,
    password: String,
}

/// Renders the login page.
#[get("/login")]
pub async fn login_page() -> Markup {
    html! {
        (base_template())
        body class="bg-gray-50 min-h-screen flex items-center justify-center py-12 px-4 sm:px-6 lg:px-8" {
            div class="max-w-md w-full space-y-8" {
                div class="text-center" {
                    h1 class="text-3xl font-bold text-gray-900 mb-2" { "Sign in to your account" }
                    p class="text-sm text-gray-600" { "Welcome back to mrkt" }
                }

                form method="post" action="/login" class="mt-8 space-y-6 bg-white p-8 rounded-lg shadow-md" {
                    div class="space-y-4" {
                        div {
                            label for="username" class="block text-sm font-medium text-gray-700 mb-1" { "Username" }
                            input
                                type="text"
                                id="username"
                                name="username"
                                required
                                class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition duration-200"
                                placeholder="Enter your username";
                        }

                        div {
                            label for="password" class="block text-sm font-medium text-gray-700 mb-1" { "Password" }
                            input
                                type="password"
                                id="password"
                                name="password"
                                required
                                class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition duration-200"
                                placeholder="Enter your password";
                        }
                    }

                    div class="pt-4" {
                        input
                            type="submit"
                            value="Sign in"
                            class="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 cursor-pointer transition duration-200";
                    }
                }

                div class="text-center mt-4" {
                    p class="text-sm text-gray-600" {
                        "Don't have an account? "
                        a href="/register" class="font-medium text-blue-600 hover:text-blue-500" {
                            "Sign up"
                        }
                    }
                }
            }
        }
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
