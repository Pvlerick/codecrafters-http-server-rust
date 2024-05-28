use std::{
    collections::HashMap,
    io::Read,
    io::Write,
    net::{TcpListener, TcpStream},
};

const RESP_200: &[u8] = b"HTTP/1.1 200 OK\r\n\r\n";
const RESP_404: &[u8] = b"HTTP/1.1 404 Not Found\r\n\r\n";

const ROOT_PATH: &[u8] = b"/";
const ECHO_PATH: &[u8] = b"/echo/";
const USERAGENT_PATH: &[u8] = b"/user-agent";

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let mut buf = [0u8; 1024];
                let read_bytes = stream.read(&mut buf).unwrap();
                if read_bytes > 0 {
                    let req = HttpRequest::parse(&buf[..read_bytes]).unwrap();
                    match (req.verb.as_ref(), req.path.as_bytes()) {
                        ("GET", ROOT_PATH) => write_response(&mut stream, RESP_200),
                        ("GET", [47, 101, 99, 104, 111, 47, content @ ..]) => {
                            write_response(&mut stream, &HttpResponse::build("200 OK", content));
                        }
                        ("GET", USERAGENT_PATH) => write_response(
                            &mut stream,
                            req.headers.get("User-Agent").unwrap().as_bytes(),
                        ),
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
    headers: HashMap<String, String>,
}

impl HttpRequest {
    fn parse(req: &[u8]) -> Option<HttpRequest> {
        let req = String::from_utf8_lossy(&req);
        let req: Vec<_> = req.split_terminator("\r\n").collect();

        let request_line = req.first().unwrap();
        let request_line: Vec<_> = request_line.split_whitespace().collect();

        let headers: HashMap<String, String, _> = HashMap::from_iter(
            req[1..req.len() - 1]
                .iter()
                .map(|i| i.split(":").collect::<Vec<_>>())
                .map(|i| (i[0].to_string(), i[1].trim().to_string())),
        );

        if request_line.len() >= 3 {
            Some(HttpRequest {
                verb: request_line[0].to_owned(),
                path: request_line[1].to_owned(),
                headers,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_request() {
        let sut = HttpRequest::parse(b"GET /user-agent HTTP/1.1\r\nHost: localhost:4221\r\nUser-Agent: foobar/1.2.3\r\nAccept: */*\r\n\r\n").unwrap();
        assert_eq!("GET", sut.verb);
        assert_eq!("/user-agent", sut.path);
        assert_eq!("localhost", sut.headers.get("Host").unwrap());
        assert_eq!("foobar/1.2.3", sut.headers.get("User-Agent").unwrap());
        assert_eq!("*/*", sut.headers.get("Accept").unwrap());
    }
}
