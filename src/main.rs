use std::{
    io::Read,
    io::Write,
    net::{TcpListener, TcpStream},
};

fn main() {
    const RESP_200: &[u8] = "HTTP/1.1 200 OK\r\n\r\n".as_bytes();
    const RESP_404: &[u8] = "HTTP/1.1 404 Not Found\r\n\r\n".as_bytes();

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let mut buf = [0u8; 1024];
                let read_bytes = stream.read(&mut buf).unwrap();
                if read_bytes > 0 {
                    let req = String::from_utf8_lossy(&buf[..read_bytes]);
                    let data: Vec<&str> = req.split_whitespace().take(2).collect();
                    match data[..] {
                        ["GET", "/"] => write_response(&mut stream, RESP_200),
                        _ => write_response(&mut stream, RESP_404),
                    }
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn write_response(mut stream: &TcpStream, response: &[u8]) {
    let _ = stream.write_all(response);
}
