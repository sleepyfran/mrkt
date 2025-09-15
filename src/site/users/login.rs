use maud::{Markup, html};
use rocket::{
    State,
    form::{Form, FromForm},
    http::{Cookie, CookieJar, Status},
    response::Redirect,
};

use crate::core::{
    auth::{LoginError, login},
    state::CoreState,
};

#[derive(FromForm)]
pub struct LoginForm {
    username: String,
    password: String,
}

#[get("/login")]
pub async fn login_page() -> Markup {
    html! {
        h1 { "Login" }
        form method="post" action="/login" {
            label for="username" { "Username:" }
            input type="text" id="username" name="username" required;
            br;
            label for="password" { "Password:" }
            input type="password" id="password" name="password" required;
            br;
            input type="submit" value="Login";
        }
    }
}

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
