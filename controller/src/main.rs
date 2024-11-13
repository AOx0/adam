pub use axum::{extract::State, routing::post, Router};
use axum::{middleware::Next, response::Response};
use deadpool::managed::Pool;
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
    println!("Request: {req:?}");
    let mut response = next.run(req).await;

    response
}

#[tokio::main]
async fn main() {
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

    let listener = TcpListener::bind("[::]:9988").await.unwrap();
    axum::serve(listener, router).await.unwrap()
}
