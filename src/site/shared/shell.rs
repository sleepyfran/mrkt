use maud::{Markup, Render, html};
use rocket::request::FlashMessage;
use serde::{Deserialize, Serialize};

/// Defines all the sections of the site that can be reached via the navigation bar.
#[derive(Debug, PartialEq, Eq)]
pub enum NavSection {
    Dashboard,
    Instruments,
    Transactions,
    Accounts,
    UserManagement,
}

/// A flash that should be handled by the shell instead of any specific page.
#[derive(Deserialize, Serialize)]
pub struct ShellFlash(String);

impl ShellFlash {
    pub fn to_flash_message(message: &str) -> String {
        let json = serde_json::to_string(&ShellFlash(message.to_string()))
            .unwrap_or_else(|_| "{}".to_string());
        format!("SHELL_FLASH:{}", json)
    }

    pub fn from_flash_message(message: &str) -> Option<Self> {
        if let Some(json_str) = message.strip_prefix("SHELL_FLASH:") {
            serde_json::from_str(json_str).ok()
        } else {
            None
        }
    }
}

/// A configurable shell for the site, which can be used to render different sections of the site.
pub struct Shell<'a> {
    /// The current section of the site that is being rendered.
    section: NavSection,

    /// Whether the header should be visible or not.
    header_visible: bool,

    /// The title of the page.
    title: &'a str,

    /// Optional flash to display if it's a shell flash.
    flash: Option<FlashMessage<'a>>,

    /// The content of the page.
    content: Markup,

    /// Extra stylesheets to be included in the page.
    extra_stylesheets: Vec<&'a str>,

    /// Extra JavaScript files to be included in the page.
    extra_scripts: Vec<&'a str>,
}

impl<'a> Shell<'a> {
    /// Creates a new shell with the given section and title.
    pub fn create(section: NavSection, title: &'a str, content: Markup) -> Self {
        Self {
            section,
            header_visible: true,
            title,
            content,
            flash: None,
            extra_stylesheets: Vec::new(),
            extra_scripts: Vec::new(),
        }
    }

    /// Hides the header of the shell. By default the header is visible.
    pub fn hide_header(mut self) -> Self {
        self.header_visible = false;
        self
    }

    /// Attaches an optional flash that will be displayed if it's a Shell Flash.
    pub fn attach_flash(mut self, flash: Option<FlashMessage<'a>>) -> Self {
        self.flash = flash;
        self
    }

    /// Adds an extra stylesheet to the shell, which gets included in the head of the page.
    pub fn add_stylesheet(mut self, stylesheet: &'a str) -> Self {
        self.extra_stylesheets.push(stylesheet);
        self
    }

    /// Adds an extra JavaScript file to the shell, which gets included at the end of the body.
    pub fn add_script(mut self, script: &'a str) -> Self {
        self.extra_scripts.push(script);
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
                @match &self.flash {
                    Some(flash) => {
                        @if let Some(shell_flash) = ShellFlash::from_flash_message(flash.message()) {
                            alert data-alert-type=(flash.kind()) {
                                p {
                                    (shell_flash.0)
                                }
                            }
                        }
                    }
                    None => { }
                }

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
                @for script in &self.extra_scripts {
                    script src={ "/public/"(script) } {}
                }
            }
        }
    }
}
