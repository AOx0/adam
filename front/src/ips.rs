use crate::{template::Template, AppState, Padded};
use axum::{
    extract::{Path, Request, State},
    http::HeaderValue,
    response::{IntoResponse, Redirect, Response},
    routing::{delete, get},
    Form, Router,
};
use front_components::*;
use maud::{html, Markup, PreEscaped};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};
use surrealdb::{opt::PatchOp, sql::Thing};

#[derive(Debug, Deserialize)]
struct AddIp {
    name: String,
    description: String,
    address: String,
    port: u16,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Ip {
    pub id: Thing,
    pub name: String,
    pub description: String,
    pub socket: SocketAddr,
    pub selected: bool,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(ips_home))
        .route("/add", get(add_ip_home).post(add_ip))
        .route("/:id", delete(delete_ip).patch(select_ip))
}

async fn select_ip(State(s): State<AppState>, Path(id): Path<String>) -> Response {
    let mut guard = s.selected_ip.write().await;

    let Some(new): Option<Ip> =
        s.db.update(("ips", &id))
            .patch(PatchOp::replace("/selected", true))
            .await
            .unwrap()
    else {
        return StatusCode::NOT_MODIFIED.into_response();
    };

    if let Some(Ip { id, .. }) = guard.as_ref()
        && *id != new.id
    {
        let _: Option<Ip> =
            s.db.update(("ips", id.id.to_string()))
                .patch(PatchOp::replace("/selected", false))
                .await
                .unwrap();
    }

    *guard = Some(new);

    let mut res = (StatusCode::OK).into_response();
    res.headers_mut()
        .insert("HX-Refresh", HeaderValue::from_static("true"));

    res
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
                    (TextInput("Name:", "text", "name", "name", true, None))
                    (TextInput("Description:", "text", "description", "description", true, None))
                    (TextInput("IP Address:", "text", "address", "address", true, None))
                    (TextInput("Port:", "number", "port", "port", true, Some("9988")))

                    (FormButton(html!{ p .font-bold { "Add IP" } }, "submit"))
                }
            }))
        })
        .await
}

async fn add_ip(State(s): State<AppState>, Form(ip): Form<AddIp>) -> impl IntoResponse {
    let Ok(ip_addr) = IpAddr::from_str(&ip.address) else {
        return ().into_response();
    };

    let socket = SocketAddr::new(ip_addr, ip.port);

    #[derive(Serialize, Deserialize)]
    pub struct IIp {
        pub name: String,
        pub description: String,
        pub socket: SocketAddr,
        pub selected: bool,
    }

    let regs: Vec<Ip> =
        s.db.insert("ips")
            .content(IIp {
                name: ip.name,
                description: ip.description,
                socket,
                selected: s.selected_ip.read().await.is_none(),
            })
            .await
            .unwrap();

    assert_eq!(regs.len(), 1);
    let data = regs.into_iter().next().unwrap();

    if s.selected_ip.read().await.is_none() {
        *s.selected_ip.write().await = Some(data);
    }

    Redirect::to("/ips").into_response()
}

// Endpoint to delete an IP
async fn delete_ip(State(s): State<AppState>, Path(id): Path<String>) -> Response {
    let mut guard = s.selected_ip.write().await;
    let _: Option<Ip> = s.db.delete(("ips", &id)).await.unwrap();

    let rem: Vec<Ip> = s.db.select("ips").await.unwrap();

    let new = rem.iter().find(|ip| ip.id.id.to_string() != id).cloned();
    let new = if let Some(new) = &new {
        s.db.update(("ips", new.id.to_string()))
            .patch(PatchOp::replace("/selected", true))
            .await
            .unwrap()
    } else {
        None
    };

    *guard = new.clone();

    let mut res = (StatusCode::OK).into_response();
    res.headers_mut()
        .insert("HX-Refresh", HeaderValue::from_static("true"));

    res
}

// Endpoint to get all IPs
async fn ips_home(templ: Template, State(state): State<AppState>) -> Markup {
    let ips: Vec<Ip> = state.db.select("ips").await.unwrap();

    templ
        .render(Padded(html! {
            script {
                (PreEscaped(r#"""
                    function select_ip(path) {
                        fetch(path, {
                            method: 'PATCH',
                        }).then(() => {
                            // location.reload();
                            window.location.href = window.location.href;
                        });
                    }
                """#))
            }

            div x-data="{ open: false }" {
                div .flex.flex-row .justify-between {
                    h1 .text-xl .font-bold { "Stored IPs" }

                    button
                        .bg-foreground.text-background
                        ."hover:bg-foreground/75"
                        .rounded
                        .px-2.py-1
                        .font-bold
                        "@click"="open = !open"
                        { "Add IP" }
                }

                table .table-auto .text-left .border-separate .w-full {
                    thead {
                        tr {
                            th .pl-8 .w-fill { "ID" }
                            th .pl-8 .w-fill { "Name" }
                            th .pl-8 .w-fill { "Description" }
                            th .pl-8 .w-fill { "Socket Addr" }
                            th .pl-8 .w-fill { "Selected" }
                            th .pl-8 .w-fill { "Action" }
                        }
                    }
                    tbody {
                        tr x-show="open" {
                            form method="post" action="/ips/add" x-ref="form" {
                                td .pl-8 { "-" }
                                td .pl-8 { input type="text" name="name" .px-1 ."bg-foreground/5" .outline-none required {} }
                                td .pl-8 { input type="text" name="description" .px-1 ."bg-foreground/5" .outline-none required {} }
                                td .pl-8 {
                                    input type="text" name="address" .px-1 .w-32 ."bg-foreground/5" .outline-none required {}
                                    input type="number" name="port" value="9988" .px-1 .w-20 .ml-2 ."bg-foreground/5" .outline-none required {}
                                }
                                td .pl-8 { "-" }
                                td .pl-8 {
                                    button
                                        .text-sm
                                        .text-foreground.transition-colors
                                        type="submit"
                                        "@click"="$refs.form.submit()"
                                    {
                                        p."hover:text-foreground/80"."text-foreground/60"
                                        { "Add" }
                                    }
                                }
                            }
                        }
                        @for ip in ips {
                            tr {
                                td .pl-8 { code { (ip.id.id) } }
                                td .pl-8 { (ip.name) }
                                td .pl-8 { (ip.description) }
                                td .pl-8 { code { (ip.socket) } }
                                td .pl-8 { @if ip.selected { "✓" } }
                                td .pl-8 .space-x-5 {
                                    button
                                        .text-sm
                                        .text-foreground.transition-colors
                                        hx-delete={ "/ips/" (ip.id.id) }
                                        hx-target="closest tr"
                                    {
                                        p."hover:text-foreground/80"."text-foreground/60"
                                        { "Delete" }
                                    }

                                    button
                                        .text-sm
                                        .text-foreground.transition-colors
                                        onclick={"select_ip(\"/ips/" (ip.id.id) "\")"}
                                        // hx-patch={ "/ips/" (ip.id.id) }


                                        // hx-swap="none"
                                        // hx-target="closest body"
                                    {
                                        p."hover:text-foreground/80"."text-foreground/60"
                                        { "Select" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }))
        .await
}
