use crate::{template::Template, AppState, Padded};
use axum::{
    extract::{Path, Request, State},
    response::{IntoResponse, Redirect},
    routing::{delete, get},
    Form, Router,
};
use front_components::*;
use maud::{html, Markup};
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

async fn select_ip(State(s): State<AppState>, Path(id): Path<String>) {
    let db = s.surrealdb.get().await.unwrap();
    let mut guard = s.selected_ip.write().await;

    let Some(new): Option<Ip> = db
        .update(("ips", &id))
        .patch(PatchOp::replace("/selected", true))
        .await
        .unwrap()
    else {
        return;
    };

    let old = guard.take();
    if let Some(Ip { id, .. }) = old {
        let _old: Option<Ip> = db
            .update(("ips", id.id.to_string()))
            .patch(PatchOp::replace("/selected", false))
            .await
            .unwrap();
    }

    *guard = Some(new);
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
                        label for="name" { "Name:" }
                        input .ml-2 type="text" id="name" name="name" required;
                    }

                    div {
                        label for="description" { "Description:" }
                        input .ml-2 type="text" id="description" name="description" required;
                    }

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

async fn add_ip(State(s): State<AppState>, Form(ip): Form<AddIp>) -> impl IntoResponse {
    let Ok(ip_addr) = IpAddr::from_str(&ip.address) else {
        return ().into_response();
    };

    let socket = SocketAddr::new(ip_addr, ip.port);
    let db = s.surrealdb.get().await.unwrap();

    #[derive(Serialize, Deserialize)]
    pub struct IIp {
        pub name: String,
        pub description: String,
        pub socket: SocketAddr,
        pub selected: bool,
    }

    let regs: Vec<Ip> = db
        .insert("ips")
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
async fn delete_ip(State(state): State<AppState>, Path(id): Path<String>) {
    let db = state.surrealdb.get().await.unwrap();

    let _: Option<Ip> = db.delete(("ips", id)).await.unwrap();
}

// Endpoint to get all IPs
async fn ips_home(templ: Template, State(state): State<AppState>) -> Markup {
    let db = state.surrealdb.get().await.unwrap();
    let ips: Vec<Ip> = db.select("ips").await.unwrap();

    templ
        .render(Padded(html! {
            div .flex .flex-col {
                a .font-bold href="/ips/add" { "Add" }
                div {
                    @for ip in ips.iter() {
                        p { (format!("{ip:?}")) }
                    }
                }
            }
        }))
        .await
}
