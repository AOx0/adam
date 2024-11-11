use axum::{extract::Path, routing::*, Router};
use front_components::Ref;
use maud::{html, Markup, PreEscaped};
use rand::RngCore;
use template::Template;
use tokio::net::TcpListener;

mod template;

#[tokio::main]
async fn main() {
    let firewall_router = Router::new()
        .route("/events", get(firewall_events))
        .route("/rules", get(rules))
        .route("/rules/:id", get(rule));

    let router = Router::new()
        .route("/", get(home))
        .nest("/firewall", firewall_router)
        .fallback(not_found);

    let listener = TcpListener::bind("[::]:8880").await.unwrap();

    axum::serve(listener, router).await.unwrap();
}

async fn not_found(templ: Template) -> Markup {
    templ.render(html! {
        div .flex .items-center .mt-20 .justify-center {
            div .text-center {
                h1 .text-6xl .font-bold .text-gray-800 { "404" }
                p .text-2xl .text-gray-600 { "Page Not Found" }
                p .mt-4 .text-gray-500 { "Sorry, the page you are looking for does not exist." }
            }
        }
    })
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
    templ.render(html! { (FirewallLog()) })
}

async fn home(templ: Template) -> Markup {
    templ.render(html! {
        "Hi"
    })
}

async fn rule(templ: Template, Path((idx,)): Path<(u32,)>) -> Markup {
    let res = reqwest::get(format!("http://127.0.0.1:9988/firewall/rules/{idx}"))
        .await
        .unwrap();

    let rule: firewall_common::StoredRuleDecoded =
        serde_json::from_str(&res.text().await.unwrap()).unwrap();

    templ.render(html! {
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
}

async fn rules(templ: Template) -> Markup {
    let res = reqwest::get("http://127.0.0.1:9988/firewall/rules")
        .await
        .unwrap();

    let rules: Vec<firewall_common::StoredRuleDecoded> =
        serde_json::from_str(&res.text().await.unwrap()).unwrap();

    let ips = vec!["192.168.1.1", "192.168.1.2", "10.0.0.1"];

    let selected_ip = "192.168.1.1";

    templ.render(html! {
        div .space-5 .m-5 {
            h1 .text-xl .font-bold { "Firewall" }

            form {
                label for="ip-select" { "Select IP address: " }
                select
                    name="ip"
                    id="ip-select"
                {
                    @for ip in ips {
                        @if ip == selected_ip {
                            option value=(ip) selected { (ip) }
                        } @else {
                            option value=(ip) { (ip) }
                        }
                    }
                }
            }

            p {
                "Status: "
                span #status
                    hx-get={"http://" (selected_ip) "/firewall/state"}
                    hx-trigger="load, every 30s, refresh"
                    hx-indicator="#loading-indicator"
                {
                    "Cargando..."
                }
                div #loading-indicator { "Actualizando..." }
            }

            script type="text/javascript" {
                (PreEscaped(r#"
                document.getElementById('ip-select').addEventListener('change', function() {
                    var selectedIp = this.value;
                    var statusSpan = document.getElementById('status');
                    statusSpan.setAttribute('hx-get', 'http://' + selectedIp + '/firewall/state');
                    // Forzar una solicitud HTMX para actualizar el estado
                    htmx.trigger(statusSpan, 'refresh');
                });
                "#))
            }

            table .table-auto .text-left .border-separate   {
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
                                    hx-delete={ "http://127.0.0.1:9988/firewall/rules/" (rule.id) }
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
    })
}
