use axum::{extract::Path, routing::*, Router};
use front_components::Ref;
use maud::{html, Markup, PreEscaped};
use template::Template;
use tokio::net::TcpListener;

mod template;

#[tokio::main]
async fn main() {
    let firewall_router = Router::new()
        .route("/rules", get(rules))
        .route("/rules/:id", get(rule));

    let router = Router::new()
        .route("/", get(home))
        .nest("/firewall", firewall_router);

    let listener = TcpListener::bind("[::]:8880").await.unwrap();

    axum::serve(listener, router).await.unwrap();
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

    templ.render(html! {
        div .space-5 .m-5 {
            h1 .text-xl .font-bold { "Firewall" }
            table .table-auto .text-left .border-separate   {
                thead {
                    tr {
                        th .pl-8 { "ID" }
                        th .pl-8 { "Name" }
                        th .pl-8 { "Description" }
                        // th .pl-8 { "Contents" }
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
                            // td .pl-8 {
                            //     code .bg-gray-100 .p-1 { (serde_json::to_string(&rule.rule).unwrap()) }
                            // }

                            td .pl-8 .text-center { (front_components::rule_status(rule.rule.enabled, rule.id as u32)) }
                            td .pl-8 .space-x-5 {
                                    (Ref("View", &format!("/firewall/rules/{}", rule.id)))

                                button
                                    hx-delete={ "http://127.0.0.1:9988/firewall/rules/" (rule.id) }
                                    hx-target="closest tr"
                                    { "Delete" }
                            }
                        }
                    }
                }
            }
        }
    })
}
