use std::{io::Write, net::TcpListener};

fn main() {
    const RESP: &[u8] = "HTTP/1.1 200 OK\r\n\r\n".as_bytes();

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let _ = stream.write_all(RESP);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
