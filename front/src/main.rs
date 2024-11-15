use axum::{
    extract::{Path, Request, State},
    response::{IntoResponse, Redirect},
    routing::*,
    Form, Router,
};
use front_components::*;
use maud::{html, Markup, PreEscaped};
use rand::RngCore;
use serde::Deserialize;
use sip::Selected;
use std::{
    net::{IpAddr, SocketAddr},
    ops::Deref,
    str::FromStr,
    sync::Arc,
};
use template::Template;
use tokio::net::TcpListener;
use tokio::sync::RwLock;

mod sip;
mod template;

// Endpoint to delete an IP
async fn delete_ip(State(state): State<AppState>, Path(id): Path<usize>) {
    let mut ips = state.inner.registered_ips.write().await;
    if id < ips.len() {
        ips.remove(id);
    }
}

// Endpoint to get all IPs
async fn ips_home(templ: Template, State(state): State<AppState>) -> Markup {
    let ips = state.registered_ips.read().await;
    let ips = ips.as_slice();

    templ
        .render(Padded(html! {
            div .flex .flex-col {
                a .font-bold href="/ips/add" { "Add" }
                div {
                    @for ip in ips.iter().enumerate() {
                        p { (format!("{ip:?}")) }
                    }
                }
            }
        }))
        .await
}

#[derive(Debug, Deserialize)]
struct AddIp {
    address: String,
    port: u16,
}

async fn add_ip(State(s): State<AppState>, Form(ip): Form<AddIp>) -> impl IntoResponse {
    let Ok(ip_addr) = IpAddr::from_str(&ip.address) else {
        return ().into_response();
    };

    let socket = SocketAddr::new(ip_addr, ip.port);
    let mut guard = s.registered_ips.write().await;
    guard.push(socket);

    if s.selected_ip.read().await.is_none() {
        *s.selected_ip.write().await = Some(socket);
    }

    drop(guard);

    Redirect::to("/ips").into_response()
}

async fn add_ip_home(templ: Template, req: Request) -> Markup {
    let err_refered = req
        .headers()
        .get("HX-Request")
        .is_some_and(|v| v.to_str().is_ok_and(|v| v == "true"));

    templ
        .render(html! {
            @if err_refered {
                (Error("You have been redirected since no IP address has been added/selected."))
            }
            (Padded(html! {
                form .flex .flex-col ."w-1/2" .space-y-2 method="post" action="/ips/add" {
                    div {
                        label for="address" { "IP Address:" }
                        input .ml-2 type="text" id="address" name="address" required;
                    }

                    div {
                        label for="port" { "Port:" }
                        input .ml-2 type="number" id="port" name="port" value="9988" required;
                    }

                    div {
                        button .font-bold type="submit" { "Add IP" }
                    }
                }
            }))
        })
        .await
}

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
    registered_ips: RwLock<Vec<SocketAddr>>,
    selected_ip: RwLock<Option<SocketAddr>>,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        inner: Arc::new(InnerState {
            registered_ips: RwLock::new(vec![]),
            selected_ip: RwLock::new(None),
        }),
    };

    let firewall_router = Router::<AppState>::new()
        .route("/events", get(firewall_events))
        .route("/rules", get(rules))
        .route("/rules/:id", get(rule));

    let ip_router = Router::new()
        .route("/", get(ips_home))
        .route("/add", get(add_ip_home).post(add_ip))
        .route("/:id", delete(delete_ip));

    let router = Router::new()
        .route("/", get(home))
        .nest("/firewall", firewall_router)
        .nest("/ips", ip_router)
        .fallback(not_found)
        .with_state(state);

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

#[allow(non_snake_case)]
fn FirewallLog(ip: SocketAddr) -> Markup {
    let mut rng = rand::thread_rng();
    let id = rng.next_u64();
    let id = format!("{id:0>21}");

    html! {
        script {
            (PreEscaped(format!("
            const ws = new WebSocket('ws://{ip}/firewall/events/ws');
            ws.onmessage = (event) => {{
                const logDiv = document.getElementById('{id}');
                const newEvent = document.createElement('p');
                newEvent.textContent = event.data;
                logDiv.appendChild(newEvent);
            }};
            ")))
        }
        div #(id) {}
    }
}

#[allow(non_snake_case)]
pub fn Padded(content: Markup) -> Markup {
    html! {
        div .space-5 .m-5 {
            (content)
        }
    }
}

async fn firewall_events(_: State<AppState>, Selected(ip): Selected, templ: Template) -> Markup {
    templ.render(html! { (FirewallLog(ip)) }).await
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

async fn rule(templ: Template, Selected(ip): Selected, Path((idx,)): Path<(u32,)>) -> Markup {
    let res = reqwest::get(format!("http://{ip}/firewall/rules/{idx}"))
        .await
        .unwrap();

    let rule: firewall_common::StoredRuleDecoded =
        serde_json::from_str(&res.text().await.unwrap()).unwrap();

    templ
        .render(Padded(html! {
            span .block .mb-5 {
                (Ref("< Rules", "/firewall/rules"))
            }

            (front_components::rule_status(rule.rule.enabled, rule.id as u32))

            h1 .text-xl .font-bold { "Rule " (rule.id) ": " (rule.name) }
            p { (rule.description) }

            code .bg-gray-100 .mt-5 .p-1 .block .whitespace-pre .overflow-x-scroll {
                (PreEscaped(serde_json::to_string_pretty(&rule.rule).unwrap()))
            }
        }))
        .await
}

async fn rules(templ: Template, Selected(ip): Selected) -> Markup {
    let res = reqwest::get(format!("http://{ip}/firewall/rules"))
        .await
        .unwrap();

    let rules: Vec<firewall_common::StoredRuleDecoded> =
        serde_json::from_str(&res.text().await.unwrap()).unwrap();

    templ.render(Padded(html! {
        h1 .text-xl .font-bold { "Firewall" }

        p { "Status: " span hx-get={"http://" (ip) "/firewall/state"} hx-trigger="load, every 30s" {} }

        table .table-auto .text-left .border-separate {
            thead {
                tr {
                    th .pl-8 { "ID" }
                    th .pl-8 { "Name" }
                    th .pl-8 { "Description" }
                    th .text-center .pl-8 { "Status" }
                    th .pl-8 { "Action" }
                }
            }
            tbody {
                @for rule in rules {
                    tr {
                        td .pl-8 { (rule.id) }
                        td .pl-8 { (rule.name) }
                        td .pl-8 { (rule.description) }
                        td .pl-8 .text-center { (front_components::rule_status(rule.rule.enabled, rule.id as u32)) }
                        td .pl-8 .space-x-5 {
                            (Ref("View", &format!("/firewall/rules/{}", rule.id)))
                            button
                                .text-sm
                                hx-delete={ "http://" (ip) "/firewall/rules/" (rule.id) }
                                hx-target="closest tr"
                            {
                                "Delete"
                            }
                        }
                    }
                }
            }
        }
    })).await
}
