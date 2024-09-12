use std::{io::Write, os::unix::net::UnixStream};

use message::{bincode, Message};

#[tokio::main]
async fn main() {
    let mut stream = UnixStream::connect("/run/adam/firewall").unwrap();

    // for i in 0..500 {
    //     stream.write_all(format!("Hi {i}\n").as_bytes()).unwrap();
    // }

    let mut buf = Vec::new();

    bincode::serialize_into(&mut buf, &Message::Terminate).unwrap();
    stream.write_all(&buf).unwrap();
}
