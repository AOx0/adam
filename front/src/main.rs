use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
    routing::*,
    Form, Router,
};
use front_components::Ref;
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
async fn delete_ip(State(state): State<AppState>, Path(ip): Path<SocketAddr>) -> impl IntoResponse {
    let mut ips = state.registered_ips.write().await;
    ips.retain(|x| x != &ip);
    "IP removed".into_response()
}

// Endpoint to get all IPs
async fn ips_home(templ: Template, State(state): State<AppState>) -> Markup {
    let ips = state.registered_ips.read().await;
    let ips = ips.as_slice();

    templ
        .render(html! {
            div {
                @for ip in ips.iter().enumerate() {
                    p { (format!("{ip:?}")) }
                }
            }
        })
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

async fn add_ip_home(templ: Template) -> Markup {
    templ
        .render(Padded(html! {
            p { "Add" }
            form method="post" action="/ips/add" {
                label for="address" { "IP Address:" }
                input type="text" id="address" name="address" required;

                label for="port" { "Port:" }
                input type="number" id="port" name="port" required;

                button type="submit" { "Add IP" }
            }
        }))
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
        .route("/:ip", delete(delete_ip));

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
            "Hi"
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
