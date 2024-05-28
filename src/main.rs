use std::{
    io::Read,
    io::Write,
    net::{TcpListener, TcpStream},
};

fn main() {
    const RESP_404: &[u8] = "HTTP/1.1 404 Not Found\r\n\r\n".as_bytes();

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let mut buf = [0u8; 1024];
                let read_bytes = stream.read(&mut buf).unwrap();
                if read_bytes > 0 {
                    let req = HttpRequest::parse(&buf[..read_bytes]).unwrap();
                    match (req.verb.as_ref(), &req.path.as_bytes()[..6]) {
                        ("GET", [47, 101, 99, 104, 111, 47]) => {
                            let content = &req.path[6..];
                            write_response(&mut stream, &HttpResponse::build("200 OK", &content));
                        }
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

struct HttpRequest {
    verb: String,
    path: String,
}

impl HttpRequest {
    fn parse(req: &[u8]) -> Option<HttpRequest> {
        let req = String::from_utf8_lossy(&req);
        let req: Vec<_> = req.split_whitespace().collect();

        if req.len() > 3 {
            Some(HttpRequest {
                verb: req[0].to_owned(),
                path: req[1].to_owned(),
            })
        } else {
            None
        }
    }
}

struct HttpResponse {}

impl HttpResponse {
    fn build<'a>(status_code: &'a str, content: &'a str) -> Vec<u8> {
        return format!(
            "HTTP/1.1 {}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
            status_code,
            content.len(),
            content
        )
        .bytes()
        .collect::<Vec<_>>();
    }
}
