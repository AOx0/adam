use std::net::SocketAddr;

use axum::{
    extract::{Path, State},
    routing::get,
    Router,
};
use front_components::*;
use maud::{html, Markup, PreEscaped};
use rand::RngCore;

use crate::{ips::Ip, sip::Selected, template::Template, AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/events", get(firewall_events))
        .route("/rules", get(rules))
        .route("/rules/:id", get(rule))
}

async fn firewall_events(_: State<AppState>, Selected(ip): Selected, templ: Template) -> Markup {
    templ.render(html! { (FirewallLog(ip.socket)) }).await
}

#[allow(non_snake_case)]
pub fn FirewallLog(ip: SocketAddr) -> Markup {
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

async fn rule(
    templ: Template,
    Selected(Ip { socket: ip, .. }): Selected,
    Path((idx,)): Path<(u32,)>,
) -> Markup {
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

            code .bg-card .mt-5 .p-1 .block .whitespace-pre .overflow-x-scroll {
                (PreEscaped(serde_json::to_string_pretty(&rule.rule).unwrap()))
            }
        }))
        .await
}

async fn rules(templ: Template, Selected(Ip { socket: ip, .. }): Selected) -> Markup {
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
                                .text-foreground.transition-colors
                                hx-delete={ "http://" (ip) "/firewall/rules/" (rule.id) }
                                hx-target="closest tr"
                            {
                                p."hover:text-foreground/80"."text-foreground/60"
                                { "Delete" }
                            }
                        }
                    }
                }
            }
        }
    })).await
}