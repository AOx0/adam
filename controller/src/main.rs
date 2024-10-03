pub use axum::{extract::State, routing::post, Router};
use axum::{middleware::Next, response::Response};
use deadpool::managed::Pool;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

mod firewall;

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
        .allow_origin([
            "http://localhost:8880"
                .parse::<axum::http::HeaderValue>()
                .unwrap(),
            "http://127.0.0.1:8880"
                .parse::<axum::http::HeaderValue>()
                .unwrap(),
        ])
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
