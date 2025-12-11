
mod request {
    use std::collections::HashMap;
    use std::net::{TcpStream};
    use std::io::Read;


    #[derive(Hash, Eq, PartialEq, Debug, Clone)]
    pub enum Method {
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
    pub struct Request {
        pub method: Method,
        pub route: String,
        pub properties: HashMap<String, String>,
        pub body: Option<Body>,
    }

    impl Request {
        pub fn new(stream: &mut TcpStream) -> Request {
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


            return Request {
                method,
                route,
                properties: hashmap,
                body,
            };
        }


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

}

pub mod response {
    pub struct Response {
        status: i8,
        content_type: Option<String>,
        content_length: Option<String>
    }

    impl Response {

    }
}

pub mod express {

    use std::net::{TcpListener};
    use std::collections::HashMap;
    use super::request::Method;
    use crate::request;
    use super::response::Response;

    type RouteFunction = dyn Fn(&request::Request , Response) -> Response + 'static;

    pub struct Application {
        methods: HashMap<(Method , String) , Box<RouteFunction>>
    }


    pub trait Server {
        fn new() -> Application;

        fn run(&mut self , port: i32);

        fn get<F>(&mut self , route: String , function: F)
        where F: Fn(&request::Request, Response) -> Response + 'static ;
        fn post<F>(&mut self , route: String , function: F)
        where F: Fn(&request::Request, Response) -> Response + 'static ;
        fn put<F>(&mut self , route: String , function: F)
        where F: Fn(&request::Request, Response) -> Response + 'static ;
        fn patch<F>(&mut self , route: String , function: F)
        where F: Fn(&request::Request, Response) -> Response + 'static ;
        fn delete<F>(&mut self , route: String , function: F)
        where F: Fn(&request::Request, Response) -> Response + 'static ;
    }

    impl Server for Application {
        fn new() -> Application {
            return Application {
                methods: HashMap::new()
            };
        }

        fn run(&mut self , port: i32) {
            let listener: TcpListener = TcpListener::bind(format!("127.0.0.1:{}" , port)).unwrap();

            println!("Started server on port {}" , port);

            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                let request = super::request::Request::new(&mut stream);
                if let Some(method) = self.methods.get(&(request.method.clone() , request.route.clone())) {
                    let f = method.as_ref();
                    f(&request , Response);
                };

                // println!("{request:#?}");
            }
        }

        fn get<F>(&mut self , route: String , function: F)
        where F: Fn(&request::Request, Response) -> Response + 'static
        {
            self.methods.insert((Method::GET , route) ,Box::new(function));
        }
        fn post<F>(&mut self , route: String , function: F)
        where F: Fn(&request::Request, Response) -> Response + 'static
        {
            self.methods.insert((Method::POST , route) ,Box::new(function));
        }
        fn put<F>(&mut self , route: String , function: F)
        where F: Fn(&request::Request, Response) -> Response + 'static
        {
            self.methods.insert((Method::PUT , route) ,Box::new(function));
        }
        fn patch<F>(&mut self , route: String , function: F)
        where F: Fn(&request::Request, Response) -> Response + 'static
        {
            self.methods.insert((Method::PATCH , route) ,Box::new(function));
        }
        fn delete<F>(&mut self , route: String , function: F)
        where F: Fn(&request::Request, Response) -> Response + 'static
        {
            self.methods.insert((Method::DELETE , route) ,Box::new(function));
        }

    }

}
