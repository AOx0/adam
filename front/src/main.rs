use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::*,
    Json, Router,
};
use front_components::Ref;
use maud::{html, Markup, PreEscaped};
use rand::RngCore;
use serde::Deserialize;
use std::{ops::Deref, sync::Arc};
use template::Template;
use tokio::net::TcpListener;
use tokio::sync::RwLock;

mod template;

// Struct for IP input
#[derive(Deserialize)]
struct IpInput {
    ip: String,
}

// Endpoint to add an IP
async fn add_ip(State(state): State<AppState>, Json(input): Json<IpInput>) -> impl IntoResponse {
    let mut ips = state.registered_ips.write().await;
    ips.push(input.ip);
    "IP added".into_response()
}

// Endpoint to delete an IP
async fn delete_ip(State(state): State<AppState>, Path(ip): Path<String>) -> impl IntoResponse {
    let mut ips = state.registered_ips.write().await;
    ips.retain(|x| x != &ip);
    "IP removed".into_response()
}

// Endpoint to get all IPs
async fn get_ips(State(state): State<AppState>) -> impl IntoResponse {
    let ips = state.registered_ips.read().await;
    Json(&*ips).into_response()
}

// Endpoint to set the selected IP
async fn set_selected_ip(
    State(state): State<AppState>,
    Json(input): Json<IpInput>,
) -> impl IntoResponse {
    *state.selected_ip.write().await = input.ip;
    "Selected IP updated".into_response()
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
    registered_ips: RwLock<Vec<String>>,
    selected_ip: RwLock<String>,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        inner: Arc::new(InnerState {
            registered_ips: RwLock::new(vec![
                "192.168.1.1".to_string(),
                "192.168.1.2".to_string(),
                "10.0.0.1".to_string(),
                "127.0.0.1".to_string(),
            ]),
            selected_ip: RwLock::new("127.0.0.1".to_string()),
        }),
    };

    let firewall_router = Router::new()
        .route("/events", get(firewall_events))
        .route("/rules", get(rules))
        .route("/rules/:id", get(rule));

    let ip_router = Router::new()
        .route("/ips", post(add_ip).get(get_ips))
        .route("/ips/selected", post(set_selected_ip))
        .route("/ips/:ip", delete(delete_ip));

    let router = Router::new()
        .route("/", get(home))
        .nest("/firewall", firewall_router)
        .nest("/api", ip_router)
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
fn FirewallLog() -> Markup {
    let mut rng = rand::thread_rng();
    let id = rng.next_u64();
    let id = format!("{id:0>21}");

    html! {
        script {
            (PreEscaped(format!("
            const ws = new WebSocket('ws://localhost:9988/firewall/events/ws');
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

async fn firewall_events(templ: Template) -> Markup {
    templ.render(html! { (FirewallLog()) }).await
}

async fn home(templ: Template) -> Markup {
    templ
        .render(html! {
            "Hi"
        })
        .await
}

async fn rule(
    templ: Template,
    Path((idx,)): Path<(u32,)>,
    State(state): State<AppState>,
) -> Markup {
    let selected_ip = state.selected_ip.read().await.clone();
    let res = reqwest::get(format!("http://{selected_ip}:9988/firewall/rules/{idx}"))
        .await
        .unwrap();

    let rule: firewall_common::StoredRuleDecoded =
        serde_json::from_str(&res.text().await.unwrap()).unwrap();

    templ
        .render(html! {
            div .space-5 .m-5 {
                span .block .mb-5 {
                    (Ref("< Rules", "/firewall/rules"))
                }

                (front_components::rule_status(rule.rule.enabled, rule.id as u32))

                h1 .text-xl .font-bold { "Rule " (rule.id) ": " (rule.name) }
                p { (rule.description) }

                code .bg-gray-100 .mt-5 .p-1 .block .whitespace-pre .overflow-x-scroll {
                    (PreEscaped(serde_json::to_string_pretty(&rule.rule).unwrap()))
                }
            }
        })
        .await
}

async fn rules(templ: Template, State(state): State<AppState>) -> Markup {
    let selected_ip = state.selected_ip.read().await.clone();

    let res = reqwest::get(format!("http://{selected_ip}:9988/firewall/rules"))
        .await
        .unwrap();

    let rules: Vec<firewall_common::StoredRuleDecoded> =
        serde_json::from_str(&res.text().await.unwrap()).unwrap();

    templ.render(html! {
        div .space-5 .m-5 {
            h1 .text-xl .font-bold { "Firewall" }

            p { "Status: " span hx-get={"http://" (selected_ip) ":9988/firewall/state"} hx-trigger="load, every 30s" {} }

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
                                    hx-delete={ "http://" (selected_ip) ":9988/firewall/rules/" (rule.id) }
                                    hx-target="closest tr"
                                {
                                    "Delete"
                                }
                            }
                        }
                    }
                }
            }
        }
    }).await
}
