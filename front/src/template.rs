use axum::http::request::Parts;
use axum::{async_trait, extract::FromRequestParts};
use front_components::Ref;
use maud::{html, Markup, PreEscaped, DOCTYPE};
use strum::{EnumIter, IntoEnumIterator};

use crate::AppState;

#[derive(Debug, Clone, Copy)]
pub enum ContentMode {
    Full,
    Embedded,
}

pub struct Template {
    title: String,
    mode: ContentMode,
    state: AppState,
}

#[allow(dead_code)]
impl Template {
    #[must_use]
    pub fn mode(&self) -> ContentMode {
        self.mode
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    #[must_use]
    pub async fn render(self, content: Markup) -> Markup {
        match self.mode {
            ContentMode::Full => {
                Template(&self.title, ContentMode::Full, content, self.state).await
            }
            ContentMode::Embedded => {
                html! {
                    head {
                        title { (self.title) }
                    }
                    (content)
                }
            }
        }
    }
}

#[async_trait]
impl FromRequestParts<AppState> for Template {
    type Rejection = ();

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Template {
            state: state.clone(),
            title: format!("ADAM - {}", parts.uri.path()),
            mode: if parts.headers.get("HX-Request").is_some() {
                ContentMode::Embedded
            } else {
                ContentMode::Full
            },
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
enum Section {
    Home,
    Firewall,
}

impl Section {
    fn map_path(self) -> &'static str {
        match self {
            Self::Home => "/",
            Self::Firewall => "/firewall",
        }
    }
}

impl maud::Render for Section {
    fn render(&self) -> Markup {
        html! {
            (format!("{:?}", self))
        }
    }
}

#[allow(clippy::too_many_lines)]
#[allow(clippy::needless_pass_by_value)]
#[allow(non_snake_case)]
async fn Template(title: &str, mode: ContentMode, content: Markup, state: AppState) -> Markup {
    if let ContentMode::Embedded = mode {
        return html! {
            (content)
        };
    }

    // Get IP list and selected IP from the global AppState
    let registered_ips = state.registered_ips.read().await.clone();
    let selected_ip = state.selected_ip.read().await.clone();

    html! {
        (DOCTYPE)
        html lang="es" {
            head {
                title { (title) }
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                script defer src="https://cdn.jsdelivr.net/npm/alpinejs@3.x.x/dist/cdn.min.js" {}
                script src="https://unpkg.com/htmx.org" {}
                script src="https://unpkg.com/htmx.org/dist/ext/response-targets.js" {}
                script src="https://unpkg.com/htmx.org/dist/ext/head-support.js" {}
                script src="https://unpkg.com/htmx.org/dist/ext/ws.js" {}
                script src="https://cdn.tailwindcss.com" {}
                script src="https://cdn.jsdelivr.net/npm/alpinejs@3.x.x/dist/cdn.min.js" defer {}
                script {
                    "
                        function toggleDarkMode() {
                            const html = document.querySelector('html');
                            const isDarkMode = html.classList.contains('dark');
                            html.classList.toggle('dark', !isDarkMode);
                            localStorage.setItem('dark', !isDarkMode);

                            return !isDarkMode;
                        }

                        function loadDarkMode() {
                            if (localStorage.getItem('dark') === null) {
                                localStorage.setItem('dark', 'true');
                            }

                            const isDarkMode = localStorage.getItem('dark') === 'true';
                            const html = document.querySelector('html');
                            html.classList.toggle('dark', isDarkMode);

                            return isDarkMode;
                        }

                        loadDarkMode();
                    "
                }
            }

            body
                hx-ext="head-support"

                .flex.flex-col.min-h-screen.relative
                .bg-background.text-foreground
                x-data="{
                    isDark: false,
                    init() {
                        this.isDark = loadDarkMode();
                    }
                }"
            {
                nav
                    .sticky."top-0"."z-50".w-full
                    .flex.flex-row.justify-between.items-center
                    ."px-6"."py-4"
                    ."border-b"."border-zinc-100/95"."dark:border-zinc-800/95"
                    .backdrop-blur
                    ."supports-[backdrop-filter]:bg-background/60"
                    ."h-[65px]"
                {
                    div.flex.flex-row.items-center."space-x-9" {

                        h1.font-semibold { "ADAM" }

                        div
                            .flex.flex-row.items-center
                            .text-sm.font-medium."space-x-4"
                            .text-foreground.transition-colors
                        {
                            @for s in Section::iter() {
                                (Ref(s, s.map_path()))
                            }
                        }
                    }

                    // IP Selector Dropdown
                    div.flex.flex-row.items-center."space-x-4" {
                        form {
                            label for="ip-select" { "Select IP: " }
                            select
                                name="ip"
                                id="ip-select"
                            {
                                @for ip in &registered_ips {
                                    @if ip == &selected_ip {
                                        option value=(ip) selected { (ip) }
                                    } @else {
                                        option value=(ip) { (ip) }
                                    }
                                }
                            }
                        }
                    }
                }

                main #main { (content) }

                (Footer())
            }

            script type="text/javascript" {
                (PreEscaped(r#"
                document.getElementById('ip-select').addEventListener('change', async function() {
                    const selectedIp = this.value;
                    await fetch('/api/ips/selected', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ ip: selectedIp })
                    });
                    // Optionally, refresh content that depends on the selected IP ? 
                });
                "#))
            }
        }
    }
}
#[allow(non_snake_case)]
fn Footer() -> Markup {
    html! {
        footer
            .flex.flex-col.mt-auto
            .bg-background
        {
            div."px-6"."py-4" {
                p.text-xl.font-bold {
                    "\u{22EF}"
                }
                p.text-xs {
                    "Made with Axum, Maud, Alpine, HTMX & Tailwind."
                }
            }
        }
    }
}
