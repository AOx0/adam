use std::sync::Arc;

pub use axum::{extract::State, routing::post, Router};
use tokio::{net::TcpListener, sync::Mutex};

mod firewall;

#[derive(Debug, Clone)]
struct AppState {
    firewall: Arc<Mutex<firewall::Socket>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            firewall: Arc::new(Mutex::new(firewall::Socket::new())),
        }
    }
}

#[tokio::main]
async fn main() {
    let state = AppState::new();

    let firewall = Router::new()
        .route("/start", post(firewall::start))
        .route("/stop", post(firewall::stop))
        .route("/halt", post(firewall::halt))
        .with_state(state);

    let router = Router::new().nest("/firewall", firewall);

    let listener = TcpListener::bind("[::]:80").await.unwrap();
    axum::serve(listener, router).await.unwrap()
}
