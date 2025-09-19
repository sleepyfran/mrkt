use maud::{Markup, html};
use time::{Date, OffsetDateTime};

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

/// Formats a date as a relative time string (e.g., "3 days ago", "2 weeks ago").
pub fn format_relative_date(date: Date) -> String {
    let today = OffsetDateTime::now_utc().date();
    let days_diff = (today - date).whole_days();

    if days_diff < 0 {
        "In the future".to_string()
    } else if days_diff == 0 {
        "Today".to_string()
    } else if days_diff == 1 {
        "Yesterday".to_string()
    } else if days_diff < 7 {
        format!("{} days ago", days_diff)
    } else if days_diff < 14 {
        "1 week ago".to_string()
    } else if days_diff < 30 {
        format!("{} weeks ago", days_diff / 7)
    } else if days_diff < 60 {
        "1 month ago".to_string()
    } else if days_diff < 365 {
        format!("{} months ago", days_diff / 30)
    } else if days_diff < 730 {
        "1 year ago".to_string()
    } else {
        format!("{} years ago", days_diff / 365)
    }
}
