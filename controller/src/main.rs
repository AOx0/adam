use std::{io::Write, os::unix::net::UnixStream, sync::Arc};

use axum::{extract::State, routing::post, Router};
use message::{bincode, Message};
use tokio::{net::TcpListener, sync::Mutex};

#[derive(Debug)]
struct FirewallController {
    buf: Vec<u8>,
    stream: UnixStream,
}

impl FirewallController {
    pub fn new() -> Self {
        Self {
            buf: Vec::with_capacity(4096),
            stream: UnixStream::connect("/run/adam/firewall").unwrap(),
        }
    }

    pub fn start(&mut self) {
        bincode::serialize_into(&mut self.buf, &Message::Start).unwrap();
        self.stream.write_all(&self.buf).unwrap();
        self.buf.clear();
    }

    pub fn halt(&mut self) {
        bincode::serialize_into(&mut self.buf, &Message::Halt).unwrap();
        self.stream.write_all(&self.buf).unwrap();
        self.buf.clear();
    }

    pub fn term(&mut self) {
        bincode::serialize_into(&mut self.buf, &Message::Terminate).unwrap();
        self.stream.write_all(&self.buf).unwrap();
        self.buf.clear();
    }
}

#[derive(Debug, Clone)]
struct AppState {
    firewall: Arc<Mutex<FirewallController>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            firewall: Arc::new(Mutex::new(FirewallController::new())),
        }
    }
}

async fn start(State(s): State<AppState>) {
    s.firewall.lock().await.start()
}

async fn stop(State(s): State<AppState>) {
    s.firewall.lock().await.term()
}

async fn halt(State(s): State<AppState>) {
    s.firewall.lock().await.halt()
}

#[tokio::main]
async fn main() {
    let state = AppState::new();
    let router = Router::new()
        .route("/start", post(start))
        .route("/stop", post(stop))
        .route("/halt", post(halt))
        .with_state(state);

    let listener = TcpListener::bind("[::]:80").await.unwrap();
    axum::serve(listener, router).await.unwrap()
}
