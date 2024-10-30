pub use axum::{extract::State, routing::post, Router};
use axum::{middleware::Next, response::Response};
use deadpool::managed::Pool;
use tokio::net::TcpListener;
use tower_http::cors::{CorsLayer, Any};

mod auth;
mod firewall;
mod surreal;
mod htmx;

const POOL_SIZE: usize = 100;

#[derive(Debug, Clone)]
struct AppState {
    firewall_pool: Pool<firewall::Manager>,
    surreal_pool: Pool<surreal::Manager>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            surreal_pool: surreal::Manager::new(POOL_SIZE),
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
        .allow_methods([axum::http::Method::POST, axum::http::Method::DELETE])
        .allow_headers([
            axum::http::HeaderName::from_static("hx-request"),
            axum::http::HeaderName::from_static("hx-current-url"),
        ]);

    let router = Router::new()
        .nest("/firewall", firewall::router())
        .layer(axum::middleware::from_fn(insert_headers))
        .layer(cors)
        .with_state(state);

    let listener = TcpListener::bind("[::]:9988").await.unwrap();
    axum::serve(listener, router).await.unwrap()
}
