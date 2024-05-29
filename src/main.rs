use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    ops::Deref,
    path::PathBuf,
    thread,
};

const RESP_200: &[u8] = b"HTTP/1.1 200 OK\r\n\r\n";
const RESP_201: &[u8] = b"HTTP/1.1 201 Created\r\n\r\n";
const RESP_404: &[u8] = b"HTTP/1.1 404 Not Found\r\n\r\n";

const ROOT_PATH: &[u8] = b"/";
// const ECHO_PATH: &[u8] = b"/echo/";
const USERAGENT_PATH: &[u8] = b"/user-agent";

fn main() {
    let args: Vec<String> = env::args().collect();
    let dir_path: Option<PathBuf> = if args.get(1).is_some_and(|i| i == "--directory") {
        Some(PathBuf::from(
            args.get(2).unwrap_or(&"".to_string()).deref(),
        ))
    } else {
        None
    };

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        let dir_path = dir_path.clone();
        thread::spawn(move || match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let mut buf = [0u8; 1024];
                let read_bytes = stream.read(&mut buf).unwrap();
                if read_bytes > 0 {
                    let req = HttpRequest::parse(&buf[..read_bytes]).unwrap();
                    match (req.verb.as_ref(), req.path.as_bytes()) {
                        ("GET", ROOT_PATH) => write_response(&mut stream, RESP_200),
                        ("GET", [47, 101, 99, 104, 111, 47, content @ ..]) => {
                            write_response(
                                &mut stream,
                                &HttpResponse::build_text_plain("200 OK", content),
                            );
                        }
                        ("GET", USERAGENT_PATH) => write_response(
                            &mut stream,
                            &HttpResponse::build_text_plain(
                                "200 OK",
                                req.headers.get("User-Agent").unwrap().as_bytes(),
                            ),
                        ),
                        ("GET", [47, 102, 105, 108, 101, 115, 47, content @ ..]) => {
                            let file_path = dir_path
                                .map(|i| i.join(String::from_utf8_lossy(content).to_string()));
                            match file_path {
                                Some(file_path) if file_path.exists() => {
                                    let mut file = File::open(file_path).unwrap();
                                    let mut buf: Vec<u8> = Vec::new();
                                    let bytes_read = file.read_to_end(&mut buf).unwrap();
                                    write_response(
                                        &mut &stream,
                                        &HttpResponse::build_octet_stream(&buf[..bytes_read]),
                                    );
                                }
                                _ => write_response(&stream, RESP_404),
                            }
                        }
                        ("POST", [47, 102, 105, 108, 101, 115, 47, content @ ..]) => {
                            let file_path = dir_path
                                .map(|i| i.join(String::from_utf8_lossy(content).to_string()));
                            match file_path {
                                Some(file_path) => {
                                    let mut file = File::create(file_path).unwrap();
                                    let _ = file.write_all(&req.body.unwrap());
                                    write_response(&mut stream, RESP_201)
                                }
                                _ => write_response(&stream, RESP_404),
                            }
                        }
                        _ => write_response(&mut stream, RESP_404),
                    }
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        });
    }
}
fn write_response(mut stream: &TcpStream, response: &[u8]) {
    let _ = stream.write_all(response);
}

struct HttpRequest {
    verb: String,
    path: String,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
}

impl HttpRequest {
    fn parse(req: &[u8]) -> Option<HttpRequest> {
        let req = String::from_utf8_lossy(&req);
        let req: Vec<_> = req.split_terminator("\r\n").collect();

        let request_line: Vec<_> = req[0].split_whitespace().collect();

        let body = if !req[req.len() - 1].is_empty() {
            Some(req[req.len() - 1].bytes().collect())
        } else {
            None
        };

        let headers: HashMap<String, String, _> = HashMap::from_iter(
            req[1..req.len() - body.as_ref().map_or(1, |_| 2)]
                .iter()
                .map(|i| i.split(":").collect::<Vec<_>>())
                .map(|i| (i[0].to_string(), i[1].trim().to_string())),
        );

        if request_line.len() >= 3 {
            Some(HttpRequest {
                verb: request_line[0].to_owned(),
                path: request_line[1].to_owned(),
                headers,
                body,
            })
        } else {
            None
        }
    }
}

struct HttpResponse {}

impl HttpResponse {
    fn build_text_plain<'a>(status_code: &'a str, content: &'a [u8]) -> Vec<u8> {
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

    fn build_octet_stream<'a>(content: &'a [u8]) -> Vec<u8> {
        let mut res = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n",
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
    fn parse_get_request() {
        let sut = HttpRequest::parse(b"GET /user-agent HTTP/1.1\r\nHost: localhost:4221\r\nUser-Agent: foobar/1.2.3\r\nAccept: */*\r\n\r\n").unwrap();
        assert_eq!("GET", sut.verb);
        assert_eq!("/user-agent", sut.path);
        assert_eq!("localhost", sut.headers.get("Host").unwrap());
        assert_eq!("foobar/1.2.3", sut.headers.get("User-Agent").unwrap());
        assert_eq!("*/*", sut.headers.get("Accept").unwrap());
        assert!(sut.body.is_none());
    }

    #[test]
    fn parse_post_request() {
        let sut = HttpRequest::parse(b"POST /user-agent HTTP/1.1\r\nHost: localhost:4221\r\nUser-Agent: foobar/1.2.3\r\nAccept: */*\r\n\r\nHello World!").unwrap();
        assert_eq!("POST", sut.verb);
        assert_eq!("/user-agent", sut.path);
        assert_eq!("localhost", sut.headers.get("Host").unwrap());
        assert_eq!("foobar/1.2.3", sut.headers.get("User-Agent").unwrap());
        assert_eq!("*/*", sut.headers.get("Accept").unwrap());
        assert_eq!("Hello World!", String::from_utf8_lossy(&sut.body.unwrap()));
    }
}
