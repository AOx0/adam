use super::*;
use axum::{extract::Path, Json};
use message::{bincode, firewall_common::FirewallRule, FirewallRequest, FirewallResponse, Message};
use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
};

#[derive(Debug, Clone)]
pub struct Manager;

impl Manager {
    pub fn new(size: usize) -> Pool<Manager> {
        Pool::builder(Manager)
            .max_size(size)
            .build()
            .expect("Failed to create UDS Pool")
    }
}

impl deadpool::managed::Manager for Manager {
    type Type = Socket;
    type Error = ();

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        Ok(Socket::new())
    }

    async fn recycle(
        &self,
        _obj: &mut Self::Type,
        _metrics: &deadpool::managed::Metrics,
    ) -> deadpool::managed::RecycleResult<Self::Error> {
        Ok(())
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/start", post(start))
        .route("/stop", post(stop))
        .route("/halt", post(halt))
        .route("/add", post(add))
        .route("/delete/:idx", post(delete))
        .route("/enable/:idx", post(enable))
        .route("/disable/:idx", post(disable))
}

#[derive(Debug)]
pub struct Socket {
    buf: Vec<u8>,
    stream: UnixStream,
}

pub async fn delete(State(s): State<AppState>, Path((idx,)): Path<(u32,)>) {
    s.firewall_pool.get().await.unwrap().delete(idx);
}

pub async fn enable(State(s): State<AppState>, Path((idx,)): Path<(u32,)>) {
    s.firewall_pool.get().await.unwrap().enable(idx);
}

pub async fn disable(State(s): State<AppState>, Path((idx,)): Path<(u32,)>) {
    s.firewall_pool.get().await.unwrap().disable(idx);
}

pub async fn add(
    State(s): State<AppState>,
    Json(rule): Json<FirewallRule>,
) -> Json<FirewallResponse> {
    let mut socket = s.firewall_pool.get().await.unwrap();
    socket.add(rule);
    Json(socket.read())
}

pub async fn start(State(s): State<AppState>) {
    s.firewall_pool.get().await.unwrap().start();
}

pub async fn stop(State(s): State<AppState>) {
    s.firewall_pool.get().await.unwrap().term();
}

pub async fn halt(State(s): State<AppState>) {
    s.firewall_pool.get().await.unwrap().halt();
}

impl Socket {
    pub fn new() -> Self {
        Self {
            buf: Vec::with_capacity(512),
            stream: UnixStream::connect("/run/adam/firewall").unwrap(),
        }
    }

    pub fn send(&mut self, msg: Message) {
        bincode::serialize_into(&mut self.buf, &msg).unwrap();
        self.stream.write_all(&self.buf).unwrap();
        self.buf.clear();
    }

    pub fn read(&mut self) -> FirewallResponse {
        bincode::deserialize_from(&self.stream).unwrap()
    }

    pub fn delete(&mut self, idx: u32) {
        self.send(Message::Firewall(FirewallRequest::DeleteRule(idx)));
    }

    pub fn enable(&mut self, idx: u32) {
        self.send(Message::Firewall(FirewallRequest::EnableRule(idx)));
    }

    pub fn disable(&mut self, idx: u32) {
        self.send(Message::Firewall(FirewallRequest::DisableRule(idx)));
    }

    pub fn add(&mut self, rule: FirewallRule) {
        self.send(Message::Firewall(FirewallRequest::AddRule(rule)))
    }

    pub fn start(&mut self) {
        self.send(Message::Start)
    }

    pub fn halt(&mut self) {
        self.send(Message::Halt)
    }

    pub fn term(&mut self) {
        self.send(Message::Terminate)
    }
}
