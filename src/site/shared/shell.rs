use maud::{Markup, Render, html};

/// Defines all the sections of the site that can be reached via the navigation bar.
#[derive(Debug, PartialEq, Eq)]
pub enum NavSection {
    Dashboard,
    Instruments,
    Transactions,
    Accounts,
    UserManagement,
}

/// A configurable shell for the site, which can be used to render different sections of the site.
pub struct Shell<'a> {
    /// The current section of the site that is being rendered.
    section: NavSection,

    /// Whether the header should be visible or not.
    header_visible: bool,

    /// The title of the page.
    title: &'a str,

    /// The content of the page.
    content: Markup,

    /// Extra stylesheets to be included in the page.
    extra_stylesheets: Vec<&'a str>,
}

impl<'a> Shell<'a> {
    /// Creates a new shell with the given section and title.
    pub fn create(section: NavSection, title: &'a str, content: Markup) -> Self {
        Self {
            section,
            header_visible: true,
            title,
            content,
            extra_stylesheets: Vec::new(),
        }
    }

    /// Hides the header of the shell. By default the header is visible.
    pub fn hide_header(mut self) -> Self {
        self.header_visible = false;
        self
    }

    /// Adds an extra stylesheet to the shell, which gets included in the head of the page.
    pub fn add_stylesheet(mut self, stylesheet: &'a str) -> Self {
        self.extra_stylesheets.push(stylesheet);
        self
    }
}

impl<'a> Render for Shell<'a> {
    fn render(&self) -> Markup {
        html! {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                link rel="stylesheet" href="/public/site.css";
                title { "mrkt // " (self.title) }
                @for stylesheet in &self.extra_stylesheets {
                    link rel="stylesheet" href={ "/public/"(stylesheet) };
                }
            }
            body {
                @if self.header_visible {
                    header {
                        h1 {
                            "mrkt"
                        }
                        nav {
                            ul {
                                li { a data-active=(self.section == NavSection::Dashboard) href="/dashboard" { "Dashboard" } }
                                li { a data-active=(self.section == NavSection::Instruments) href="/instruments" { "Instruments" } }
                                li { a data-active=(self.section == NavSection::Transactions) href="/transactions" { "Transactions" } }
                                li { a data-active=(self.section == NavSection::Accounts) href="/accounts" { "Accounts" } }
                            }
                        }
                    }
                }
                main {
                    (self.content)
                }
            }
        }
    }
}
