#![feature(let_chains)]

use axum::middleware::Next;
use axum::response::Response;
use axum::{routing::*, Router};
use front_components::*;
use ips::Ip;
use log::info;
use maud::{html, Markup};
use std::{ops::Deref, sync::Arc};
use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::Surreal;
use template::Template;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tower_http::services::ServeDir;

mod firewall;
mod ips;
mod sip;
mod template;

impl Deref for AppState {
    type Target = InnerState;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug, Clone)]
struct AppState {
    inner: Arc<InnerState>,
}

#[derive(Debug)]
struct InnerState {
    selected_ip: RwLock<Option<Ip>>,
    db: Surreal<Db>,
}

#[allow(clippy::let_and_return, unused_mut)]
pub async fn insert_headers(req: axum::extract::Request, next: Next) -> Response {
    info!("{} {}", req.method(), req.uri());
    let mut response = next.run(req).await;

    response
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let storage = dirs::data_local_dir().unwrap().join("adam");
    let storage = storage.as_path();
    std::fs::create_dir_all(storage).unwrap();
    let db = surrealdb::Surreal::new::<RocksDb>(storage).await.unwrap();
    db.use_ns("adam").use_db("adam").await.unwrap();

    let selected_ip = {
        let ips: Vec<Ip> = db.select("ips").await.unwrap();
        ips.into_iter().find(|ip| ip.selected)
    };

    let state = AppState {
        inner: Arc::new(InnerState {
            selected_ip: RwLock::new(selected_ip),
            db,
        }),
    };

    let router = Router::new()
        .route("/", get(home))
        .nest("/firewall", firewall::router())
        .nest("/ips", ips::router())
        .layer(axum::middleware::from_fn(insert_headers))
        .fallback(not_found)
        .fallback_service(ServeDir::new("./front/static/"))
        .with_state(state);

    let uri = "http://[::]:8880".to_string();
    let link = terminal_link::Link::new(&uri, &uri);
    info!("Available on {link}");

    let listener = TcpListener::bind("[::]:8880").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

async fn not_found(templ: Template) -> Markup {
    templ
        .render(html! {
            div .flex .items-center .mt-20 .justify-center {
                div .text-center {
                    h1 .text-6xl .font-bold .text-gray-800 { "404" }
                    p .text-2xl .text-gray-600 { "Page Not Found" }
                    p .mt-4 .text-gray-500 { "Sorry, the page you are looking for does not exist." }
                }
            }
        })
        .await
}

async fn home(templ: Template) -> Markup {
    templ
        .render(Padded(html! {
            div.space-5.m-5 style="font-size:80px; font-weight:bold" {
                h1.txt-xl {
                    span style="font-weight:900" { "A" }
                    "gent-based "
                    span style="font-weight:900" { "D" }
                    "evice "
                    span style="font-weight:900" { "A" }
                    "udit "
                    span style="font-weight:900" { "M" }
                    "onitor"
                }
            }
            div.grafana-stack.p-8  {
                // Header - full width
                div.text-center.mb-8 {
                    h1.text-2xl.text-white { "Understand your Resources and Optimize" }
                    h1.text-2xl.text-white { "using some, or all pluggins" }
                }

                // Content wrapper with grid
                div.flex style="gap: 2rem" {
                    // Left column - about 25% width
                    div.flex-col style="gap: 1rem; width: 25%" {
                        // Tools box
                        div.rounded style="background-color: #2E2E2B; padding: 1rem; text-align: center;" {
                            h2.text-white { "Your tools / data" }
                            // Grid for icons would go here
                        }

                        // Environment box
                        div.rounded style="background-color: #2E2E2B; padding: 1rem; text-align: center;" {
                            h2.text-white { "Your environment" }
                            p.text-gray-300 { "Applications and infrastructure" }
                        }
                    }

                    // Right column - about 75% width
                    div.flex-col style="gap: 1rem; width: 75%" {
                        // Visualization - full width
                        div.rounded style="background-color: #9090A0; padding: 1rem; text-align: center;" {
                            h2 { "Visualization" }
                        }

                        // Three equal boxes row
                        div.grid style="grid-template-columns: repeat(3, 1fr); gap: 1rem" {
                            div.rounded style="background-color: #9B7EDE; padding: 1rem; text-align: center;" {
                                h3 { "Testing" }
                            }
                            div.rounded style="background-color: #5B9BD5; padding: 1rem; text-align: center;" {
                                h3 { "Observability solutions" }
                            }
                            div.rounded style="background-color: #68B37E; padding: 1rem; text-align: center;" {
                                h3 { "Incident response management" }
                            }
                        }

                        // Key capabilities - full width
                        div.rounded style="background-color: #FF7F50; padding: 1rem; text-align: center;" {
                            h3 { "Key capabilities" }
                        }

                        // Building blocks - full width
                        div.rounded style="background-color: #FFB366; padding: 1rem; text-align: center;" {
                            h3 { "Building blocks: telemetry databases (LGTM+)" }
                        }
                    }
                }
            }
        }))
        .await
}
