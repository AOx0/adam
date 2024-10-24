use axum::{
    extract::{ws::WebSocket, Path, State, WebSocketUpgrade},
    response::Response,
    routing::{get, post},
    Json, Router,
};
use deadpool::managed::Pool;
use futures::{SinkExt, StreamExt};
use message::{
    async_bincode::{tokio::AsyncBincodeStream, AsyncDestination},
    firewall,
    firewall_common::{Event, StoredRuleDecoded},
    Message,
};
use tokio::net::UnixStream;

use crate::AppState;

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
        Ok(Socket::new().await)
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
        .route("/toggle/:idx", post(toggle))
        .route("/rule/:idx", get(get_rule))
        .route("/rules", get(get_rules))
        .route("/events", get(listen_events))
        .route("/status", get(status))
}

#[derive(Debug)]
pub struct Socket {
    stream: AsyncBincodeStream<UnixStream, firewall::Response, Message, AsyncDestination>,
}

pub async fn status(State(state): State<AppState>) -> Json<firewall::Status> {
    Json(state.firewall_pool.get().await.unwrap().status().await)
}

pub async fn event_dispatcher(mut socket: WebSocket) {
    use message::async_bincode::tokio::AsyncBincodeReader;

    let uds = UnixStream::connect("/run/adam/firewall_events")
        .await
        .unwrap();
    let mut uds: AsyncBincodeReader<UnixStream, Event> = AsyncBincodeReader::from(uds);

    loop {
        let Ok(event): Result<Event, _> = futures::StreamExt::next(&mut uds).await.unwrap() else {
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
    s.firewall_pool.get().await.unwrap().delete(idx).await;
}

pub async fn enable(State(s): State<AppState>, Path((idx,)): Path<(u32,)>) {
    s.firewall_pool.get().await.unwrap().enable(idx).await;
}

pub async fn disable(State(s): State<AppState>, Path((idx,)): Path<(u32,)>) {
    s.firewall_pool.get().await.unwrap().disable(idx).await;
}

pub async fn toggle(State(s): State<AppState>, Path((idx,)): Path<(u32,)>) {
    s.firewall_pool.get().await.unwrap().toggle(idx).await;
}

pub async fn get_rule(
    State(s): State<AppState>,
    Path((idx,)): Path<(u32,)>,
) -> Json<Option<StoredRuleDecoded>> {
    Json(s.firewall_pool.get().await.unwrap().get_rule(idx).await)
}

pub async fn get_rules(State(s): State<AppState>) -> Json<Vec<StoredRuleDecoded>> {
    Json(s.firewall_pool.get().await.unwrap().get_rules().await)
}

pub async fn add(
    State(s): State<AppState>,
    Json(rule): Json<StoredRuleDecoded>,
) -> Json<firewall::Response> {
    let mut socket = s.firewall_pool.get().await.unwrap();
    socket.add(rule).await;
    Json(socket.read().await)
}

pub async fn start(State(s): State<AppState>) {
    s.firewall_pool.get().await.unwrap().start().await;
}

pub async fn stop(State(s): State<AppState>) {
    s.firewall_pool.get().await.unwrap().term().await;
}

pub async fn halt(State(s): State<AppState>) {
    s.firewall_pool.get().await.unwrap().halt().await;
}

impl Socket {
    pub async fn new() -> Self {
        let stream: AsyncBincodeStream<UnixStream, firewall::Response, Message, AsyncDestination> =
            AsyncBincodeStream::from(UnixStream::connect("/run/adam/firewall").await.unwrap())
                .for_async();

        Self { stream }
    }

    pub async fn send(&mut self, msg: Message) {
        self.stream.send(msg).await.unwrap();
    }

    pub async fn read(&mut self) -> firewall::Response {
        self.stream.next().await.unwrap().unwrap()
    }

    pub async fn delete(&mut self, idx: u32) {
        self.send(Message::Firewall(firewall::Request::DeleteRule(idx)))
            .await;
    }

    pub async fn enable(&mut self, idx: u32) {
        self.send(Message::Firewall(firewall::Request::EnableRule(idx)))
            .await;
    }

    pub async fn disable(&mut self, idx: u32) {
        self.send(Message::Firewall(firewall::Request::DisableRule(idx)))
            .await;
    }

    pub async fn toggle(&mut self, idx: u32) {
        self.send(Message::Firewall(firewall::Request::ToggleRule(idx)))
            .await;
    }

    pub async fn add(&mut self, rule: StoredRuleDecoded) {
        self.send(Message::Firewall(firewall::Request::AddRule(rule)))
            .await
    }

    pub async fn status(&mut self) -> firewall::Status {
        self.send(Message::Firewall(firewall::Request::Status))
            .await;
        let read = self.read().await;
        let firewall::Response::Status(status) = read else {
            unreachable!("It should always");
        };

        status
    }

    pub async fn get_rule(&mut self, idx: u32) -> Option<StoredRuleDecoded> {
        self.send(Message::Firewall(firewall::Request::GetRule(idx)))
            .await;
        let read = self.read().await;
        if let firewall::Response::Rule(rule) = read {
            Some(rule)
        } else {
            None
        }
    }

    pub async fn get_rules(&mut self) -> Vec<StoredRuleDecoded> {
        self.send(Message::Firewall(firewall::Request::GetRules))
            .await;

        match self.read().await {
            firewall::Response::Rules(rules) => rules,
            firewall::Response::DoesNotExist => vec![],
            firewall::Response::Id(_) => unreachable!(),
            firewall::Response::Rule(_) => unreachable!(),
            firewall::Response::ListFull => unreachable!(),
            firewall::Response::Status(_) => unreachable!(),
        }
    }

    pub async fn start(&mut self) {
        self.send(Message::Start).await
    }

    pub async fn halt(&mut self) {
        self.send(Message::Halt).await
    }

    pub async fn term(&mut self) {
        self.send(Message::Terminate).await
    }
}
