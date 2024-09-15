use super::*;
use message::{bincode, Message};
use std::{io::Write, os::unix::net::UnixStream};

#[derive(Debug)]
pub struct Socket {
    buf: Vec<u8>,
    stream: UnixStream,
}

pub async fn start(State(s): State<AppState>) {
    s.firewall.lock().await.start()
}

pub async fn stop(State(s): State<AppState>) {
    s.firewall.lock().await.term()
}

pub async fn halt(State(s): State<AppState>) {
    s.firewall.lock().await.halt()
}

impl Socket {
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
