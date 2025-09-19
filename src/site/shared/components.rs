use maud::{Markup, html};

/// Renders an empty state component with a title, message, and action button.
pub fn empty_state(
    icon: &str,
    title: &str,
    message: &str,
    action: &str,
    action_url: &str,
) -> Markup {
    html! {
        empty-state {
            empty-state-content-wrapper {
                empty-state-icon { (icon) }
                empty-state-title { (title) }
                empty-state-message { (message) }
                empty-state-action {
                    a href=(action_url) { (action) }
                }
            }
        }
    }
}
