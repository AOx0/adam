use axum::{extract::Request, middleware::Next, response::Response};
pub use axum::{extract::State, routing::post, Router};
use deadpool::managed::Pool;
use tokio::net::TcpListener;

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

pub async fn insert_cors(req: Request, next: Next) -> Response {
    println!("Request: {req:?}");
    let mut response = next.run(req).await;

    response.headers_mut().insert(
        "Access-Control-Allow-Origin",
        "*".parse().expect("Infallible"),
    );

    response
}

#[tokio::main]
async fn main() {
    let state = AppState::new();

    let router = Router::new()
        .nest("/firewall", firewall::router())
        .route_layer(axum::middleware::from_fn(insert_cors))
        .with_state(state);

    let listener = TcpListener::bind("[::]:80").await.unwrap();
    axum::serve(listener, router).await.unwrap()
}
