use super::*;
use axum::{
    extract::{ws::WebSocket, Path, WebSocketUpgrade},
    response::Response,
    routing::get,
    Json,
};
use message::{
    bincode,
    firewall_common::{FirewallEvent, FirewallRule},
    FirewallRequest, FirewallResponse, Message,
};
use std::{io::Write, os::unix::net::UnixStream};

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
        .route("/rule/:idx", get(get_rule))
        .route("/rules", get(get_rules))
        .route("/events", get(listen_events))
}

#[derive(Debug)]
pub struct Socket {
    buf: Vec<u8>,
    stream: UnixStream,
}

pub async fn event_dispatcher(mut socket: WebSocket) {
    let mut uds = UnixStream::connect("/run/adam/firewall_events").unwrap();

    loop {
        let Ok(event): Result<FirewallEvent, _> = bincode::deserialize_from(&mut uds) else {
            break; // If it fails it may be that the firewall stopped
        };

        let Ok(_) = socket
            .send(axum::extract::ws::Message::Text(
                serde_json::to_string(&event).unwrap(),
            ))
            .await
        else {
            break; // We will just drop de connection if it fails
        };
    }
}

pub async fn listen_events(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(event_dispatcher)
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

pub async fn get_rule(
    State(s): State<AppState>,
    Path((idx,)): Path<(u32,)>,
) -> Json<Option<FirewallRule>> {
    Json(s.firewall_pool.get().await.unwrap().get_rule(idx))
}

pub async fn get_rules(State(s): State<AppState>) -> Json<Vec<FirewallRule>> {
    Json(s.firewall_pool.get().await.unwrap().get_rules())
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
            buf: Vec::with_capacity(2048),
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

    pub fn get_rule(&mut self, idx: u32) -> Option<FirewallRule> {
        self.send(Message::Firewall(FirewallRequest::GetRule(idx)));
        let read = self.read();
        if let FirewallResponse::Rule(rule) = read {
            Some(rule)
        } else {
            None
        }
    }

    pub fn get_rules(&mut self) -> Vec<FirewallRule> {
        self.send(Message::Firewall(FirewallRequest::GetRules));

        match self.read() {
            FirewallResponse::Rules(rules) => rules,
            FirewallResponse::DoesNotExist => vec![],
            FirewallResponse::Id(_) => unreachable!(),
            FirewallResponse::Rule(_) => unreachable!(),
            FirewallResponse::ListFull => unreachable!(),
        }
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
