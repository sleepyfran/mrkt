use maud::{Markup, html};

/// Returns the base HTML template for the site, including the head section with
/// meta tags and the link to the compiled Tailwind CSS file.
pub fn base_template() -> Markup {
    html! {
      head {
        meta charset="utf-8";
        meta name="viewport" content="width=device-width, initial-scale=1";
        link rel="stylesheet" href="/public/site.css";
        title { "mrkt" }
      }
    }
}
