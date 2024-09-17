use super::*;
use message::{bincode, Message};
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
}

#[derive(Debug)]
pub struct Socket {
    buf: Vec<u8>,
    stream: UnixStream,
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
