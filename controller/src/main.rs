use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

pub use axum::{extract::State, routing::post, Router};
use axum::{middleware::Next, response::Response};
use clap::Parser;
use deadpool::managed::Pool;
use log::info;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

mod firewall;
mod htmx;

const POOL_SIZE: usize = 100;

#[derive(Debug, Clone)]
struct AppState {
    firewall_pool: Pool<firewall::Manager>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            firewall_pool: firewall::Manager::new(POOL_SIZE),
        }
    }
}

#[allow(clippy::let_and_return, unused_mut)]
pub async fn insert_headers(req: axum::extract::Request, next: Next) -> Response {
    info!("{} {}", req.method(), req.uri());
    let mut response = next.run(req).await;

    response
}

#[derive(Debug, Parser)]
struct Args {
    #[clap(long, short, alias = "addr")]
    address: Option<IpAddr>,
    #[clap(long, short)]
    port: Option<u16>,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let Args { address, port } = Args::parse();
    let address = address.unwrap_or(IpAddr::from_str("::").expect("Should not fail"));
    let port = port.unwrap_or(9988);

    let socket = SocketAddr::new(address, port);
    info!("Binding to {socket:?}");

    let state = AppState::new();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_private_network(true);

    let router = Router::new()
        .nest("/firewall", firewall::router())
        .layer(axum::middleware::from_fn(insert_headers))
        .layer(cors)
        .with_state(state);

    let listener = TcpListener::bind(socket).await.unwrap();
    axum::serve(listener, router).await.unwrap()
}
