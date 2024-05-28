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
                    let req = HttpRequest::parse(&buf[..read_bytes]).unwrap();
                    match (req.verb.as_ref(), &req.path.as_bytes()) {
                        ("GET", [47]) => write_response(&mut stream, RESP_200),
                        ("GET", [47, 101, 99, 104, 111, 47, content @ ..]) => {
                            write_response(&mut stream, &HttpResponse::build("200 OK", content));
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
    fn build<'a>(status_code: &'a str, content: &'a [u8]) -> Vec<u8> {
        let mut res = format!(
            "HTTP/1.1 {}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n",
            status_code,
            content.len(),
        )
        .bytes()
        .collect::<Vec<_>>();
        res.extend_from_slice(content);

        res
    }
}
