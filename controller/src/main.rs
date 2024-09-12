use std::{io::Write, os::unix::net::UnixStream};

#[tokio::main]
async fn main() {
    let mut stream = UnixStream::connect("/run/adam/firewall").unwrap();

    for i in 0..500 {
        stream.write_all(format!("Hi {i}\n").as_bytes()).unwrap();
    }

    stream.write_all(b"exit").unwrap();
}
