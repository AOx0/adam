use axum::{routing::*, Router};
use maud::{html, Markup};
use template::Template;
use tokio::net::TcpListener;

mod template;

#[tokio::main]
async fn main() {
    let router = Router::new().route("/", get(home));

    let listener = TcpListener::bind("[::]:8880").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

async fn home(templ: Template) -> Markup {
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
                        th .pl-8 { "Contents" }
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
                            td .pl-8 {
                                code .bg-gray-100 .p-1 { (serde_json::to_string(&rule.rule).unwrap()) }
                            }

                            td .pl-8 .text-center {
                                @let color = { if rule.rule.enabled { "lime" } else { "rose" } };
                                @let text = { if rule.rule.enabled { "Enabled" } else { "Disabled" } };
                                @let enabled = { if rule.rule.enabled { "true" } else { "false" } };

                                button
                                    x-data={"{ enabled: " (enabled) " }"}
                                    "x-on:click"="enabled = !enabled;" 
                                    hx-post={ "http://127.0.0.1:9988/firewall/toggle/" (rule.id) }
                                    hx-swap="none"
                                    .{ "bg-"(color)"-200" }
                                    .{ "border-"(color)"-800" }
                                    .{ "text-"(color)"-800" }
                                    .border.rounded-full
                                    .px-2
                                    { (text) }
                            }
                            td .pl-8 {
                                button
                                    hx-delete={ "http://127.0.0.1:9988/firewall/delete/" (rule.id) }
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
