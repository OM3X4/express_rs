use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use std::{fs, thread};
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug)]
enum Method {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
}

#[derive(Debug)]
enum Body {
    JSON(String),
    FormData(HashMap<String, String>),
    Text(String),
    Binary(Vec<u8>),
}

#[derive(Debug)]
struct Request {
    method: Method,
    route: String,
    properties: HashMap<String, String>,
    body: Option<Body>,
}

impl Request {
    fn new(stream: &mut TcpStream) -> Request {
        let (v, left_over_of_body) = read_header(stream);

        let first_line: Vec<_> = v[0]
            .split_ascii_whitespace()
            .map(|item| item.trim())
            .filter(|item| !item.is_empty())
            .collect();

        if first_line.len() != 3 {
            panic!("The Header is invalid (first line)")
        }

        let method = match first_line[0] {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "PATCH" => Method::PATCH,
            "DELETE" => Method::DELETE,
            _ => panic!("Invalid Method"),
        };
        let route = first_line[1].to_string();
        let mut hashmap = HashMap::new();
        for i in &v[1..] {
            if let Some((name, value)) = i.split_once(":") {
                let name = name.trim().to_string();
                let value = value.trim().to_string();

                hashmap
                    .entry(name)
                    .and_modify(|v: &mut String| {
                        v.push_str(", ");
                        v.push_str(&value);
                    })
                    .or_insert(value);
            }
        }

        let body: Option<Body>;

        if let (Some(content_length), Some(content_type)) =
            (hashmap.get("Content-Length"), hashmap.get("Content-Type"))
        {
            let body_bytes = read_body(stream, content_length.parse().unwrap(), left_over_of_body);
            body = match content_type.as_str() {
                "application/json" => {
                    Some(Body::JSON(String::from_utf8_lossy(&body_bytes).to_string()))
                }
                "application/x-www-form-urlencoded" => {
                    let mut map = HashMap::new();
                    let string = String::from_utf8_lossy(&body_bytes).to_string();
                    let vec: Vec<&str> = string.split("&").collect();
                    for key_value in vec {
                        let (key, value) = key_value.split_once("=").unwrap();
                        map.insert(key.to_string(), value.to_string());
                    }
                    Some(Body::FormData(map))
                }
                "text/plain" => Some(Body::Text(String::from_utf8_lossy(&body_bytes).to_string())),
                x if !x.is_empty() => Some(Body::Binary(body_bytes)),
                _ => None,
            };
        } else {
            body = None
        }

        std::thread::sleep(Duration::from_secs(10));

        return Request {
            method,
            route,
            properties: hashmap,
            body,
        };
    }
}

fn main() {
    // let listener: TcpListener = TcpListener::bind("127.0.0.1:7878").unwrap();
    // let thread_num = Arc::new(Mutex::new(0));

    // for stream in listener.incoming() {
    //     let thread_num = Arc::clone(&thread_num);
    //     if *thread_num.lock().unwrap() < 2 {
    //         thread::spawn(move || {
    //             *thread_num.lock().unwrap() += 1;
    //             let mut stream = stream.unwrap();
    //             let header = Request::new(&mut stream);
    //             println!("{:#?} {}", header.method, header.route);
    //             let _ = stream.write_all("my name is omar".as_bytes());

    //             let status_line = "HTTP/1.1 200 OK";
    //             let contents = fs::read_to_string("index.html").unwrap();
    //             let length = contents.len();

    //             let response = format!(
    //                 "{status_line}\r\nContent-Length: {length}\r\nContent-Type: text/html\r\n\r\n{contents}"
    //             );
    //             stream.write_all(response.as_bytes()).unwrap();
    //             *thread_num.lock().unwrap() -= 1;
    //         });
    //     } else {
    //         let mut stream = stream.unwrap();
    //         let header = Request::new(&mut stream);
    //         println!("{:#?} {}", header.method, header.route);
    //         let _ = stream.write_all("my name is omar".as_bytes());

    //         let status_line = "HTTP/1.1 200 OK";
    //         let contents = fs::read_to_string("index.html").unwrap();
    //         let length = contents.len();

    //         let response = format!(
    //             "{status_line}\r\nContent-Length: {length}\r\nContent-Type: text/html\r\n\r\n{contents}"
    //         );
    //         stream.write_all(response.as_bytes()).unwrap();
    //     }
    // }


    use express_rs::express;
    use express::Server;

    let mut server =  express::Application::new();

    server.get(String::from("/hello"), |req , _res| {

        println!("The Request is \n{:#?}" , req);


        return express_rs::response::Response;
    });
    server.put(String::from("/omar") , |req , _res| {
        println!("The Request is \n{:#?}" , req);
        return _res;
    });

    server.run(7878)

}

fn read_body(stream: &mut TcpStream, content_length: usize, left_over: Vec<u8>) -> Vec<u8> {
    let mut buf = left_over;
    let mut rest = vec![0u8; content_length - buf.len()];
    let _ = stream.read_exact(&mut rest);
    buf.extend_from_slice(&rest);
    return buf;
}

fn read_header(stream: &mut TcpStream) -> (Vec<String>, Vec<u8>) {
    let mut buf: Vec<_> = Vec::new();
    let mut temp = [0u8; 512];

    loop {
        let n = stream.read(&mut temp).unwrap();
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&temp[..n]);

        if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            let header_end = 4 + pos;
            let header_bytes = buf[..header_end].to_vec();
            let left_over = buf[header_end..].to_vec();

            let header = String::from_utf8_lossy(&header_bytes).to_string();
            let vec: Vec<_> = header
                .split("\n")
                .map(|line| line.trim().to_owned())
                .filter(|line| !line.is_empty())
                .collect();

            return (vec, left_over);
        }
    }

    panic!("Connection Stopped before finishing")
}
