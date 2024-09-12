use std::{io::Write, os::unix::net::UnixStream, time::Duration};

use message::{bincode, Message};
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    let mut stream = UnixStream::connect("/run/adam/firewall").unwrap();

    // for i in 0..500 {
    //     stream.write_all(format!("Hi {i}\n").as_bytes()).unwrap();
    // }

    println!("Message::Start");
    let mut buf = Vec::new();
    bincode::serialize_into(&mut buf, &Message::Start).unwrap();
    stream.write_all(&buf).unwrap();
    println!("Message::Start sent");

    sleep(Duration::from_secs(5)).await;

    println!("Message::Terminate");
    let mut buf = Vec::new();
    bincode::serialize_into(&mut buf, &Message::Terminate).unwrap();
    stream.write_all(&buf).unwrap();
    println!("Message::Terminate sent");
}
