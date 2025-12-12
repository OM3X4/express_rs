//! Express rs is a small simple web server library similar to express js
//!
//! Example:
//! ```rust
//! let app = express::Application::new(); \
//! app.get("/hello" , |req , res| { \
//!     return res.status(200).json(r#"{"name":"omar"}"#)) \
//! })
//! ```

pub mod express {
    use std::collections::HashMap;
    use std::io::Read;
    use std::io::Write;
    use std::net::{TcpListener, TcpStream};

    /// This enum define the fundmental HTTP Methods (GET, POST , PUT , PATCH , DELETE)
    #[derive(Hash, Eq, PartialEq, Debug, Clone)]
    pub enum Method {
        GET,
        POST,
        PUT,
        PATCH,
        DELETE,
    }

    /// This enum is for request body parsing , it contain fundmental types (JSON , FormData , Text , Binary)
    #[derive(Debug)]
    pub enum Body {
        JSON(String),
        FormData(HashMap<String, String>),
        Text(String),
        Binary(Vec<u8>),
    }

    /// Request struct is responsible for incoming request parsing
    #[derive(Debug)]
    pub struct Request {
        /// The used method in the request , an instance of [Method] Enum
        pub method: Method,
        /// The route of the request
        pub route: String,
        /// The headers of the request
        pub headers: HashMap<String, String>,
        /// The body of the request , an instance of [Body] Enum
        pub body: Option<Body>,
        params: Option<HashMap<String, String>>,
        search_params: Option<HashMap<String, String>>,
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
                let body_bytes =
                    read_body(stream, content_length.parse().unwrap(), left_over_of_body);
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
                    "text/plain" => {
                        Some(Body::Text(String::from_utf8_lossy(&body_bytes).to_string()))
                    }
                    x if !x.is_empty() => Some(Body::Binary(body_bytes)),
                    _ => None,
                };
            } else {
                body = None
            }

            return Request {
                method,
                route,
                headers: hashmap,
                body,
                params: None,
                search_params: None,
            };
        }

        /// This function is used to get a param from the request \
        /// It uses the dynamic route defined in the method definition
        ///
        /// # Example:
        /// ```rust
        ///    app.get("/user/:id", |request, response| { \
        ///        let id = request.get_param("id").unwrap(); \
        ///         response.status(200).json(format!(r#"{"id": {}}"# , id.parse().unwrap())); \
        ///    });
        /// ```
        ///
        pub fn get_param(&self, key: &str) -> Option<String> {
            match &self.params {
                Some(map) => match map.get(key) {
                    Some(value) => Some(value.clone()),
                    None => None,
                },
                None => None,
            }
        }
        /// This function is used to get a search_param from the request
        ///
        /// # Example:
        /// ```rust
        ///    app.get("/products", |request, response| {
        ///        let max_price = request.get_search_param("max_price").unwrap();
        ///        response.status(200);
        ///    });
        /// ```
        ///
        pub fn get_search_param(&self, key: &str) -> Option<String> {
            match &self.search_params {
                Some(map) => match map.get(key) {
                    Some(value) => Some(value.clone()),
                    None => None,
                },
                None => None,
            }
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


    /// The Response struct is used to send a response to the client
    /// It can be custom
    pub struct Response {
        status: i32,
        content_type: Option<String>,
        content_length: Option<i32>,
        body: String,
    }

    impl Response {
        fn new() -> Response {
            Response {
                status: 200,
                content_length: None,
                content_type: None,
                body: "".to_string(),
            }
        }
        /// A function to set the status code of the response
        ///
        /// It returns the response object , so it can be chained
        ///
        /// # Example:
        /// ```rust
        ///    app.get("/hello", |request, response| {
        ///        response.status(200);
        ///    });
        /// ```
        ///
        pub fn status(mut self, code: i32) -> Self {
            self.status = code;
            return self;
        }
        /// A function to set the body of the response to HTML
        ///
        /// It returns the response object , so it can be chained
        ///
        /// # Example:
        /// ```rust
        ///    app.get("/hello", |request, response| {
        ///        response.status(200).html("<h1>Hello World</h1>".to_string());
        ///    });
        /// ```
        ///
        pub fn html(mut self, html: String) -> Self {
            self.content_type = Some("text/html".to_string());
            self.content_length = Some(html.len() as i32);
            self.body = html;
            return self;
        }
        /// A function to set the body of the response to JSON
        ///
        /// It returns the response object , so it can be chained
        ///
        /// # Example:
        /// ```rust
        ///    app.get("/hello", |request, response| {
        ///        response.status(200).json(r#"{"message": "Hello World"}"#);
        ///    });
        /// ```
        ///
        pub fn json(mut self, json: String) -> Self {
            self.content_type = Some("application/json".to_string());
            self.content_length = Some(json.len() as i32);
            self.body = json;
            return self;
        }
        fn send(&mut self, stream: &mut TcpStream) {
            let status_line = format!("HTTP/1.1 {}", self.status);
            println!("");
            if let (Some(content_len), Some(content_type)) =
                (&self.content_length, &self.content_type)
            {
                let response = format!(
                    "{status_line}\r\nContent-Length: {}\r\nContent-Type: {}\r\n\r\n{}",
                    content_len, content_type, self.body
                );
                stream.write_all(response.as_bytes()).unwrap();
            } else {
                let response = format!("{status_line}\r\n\r\n");
                stream.write_all(response.as_bytes()).unwrap();
            }
        }
    }

    #[derive(Debug)]
    enum RouteSegment {
        Static(String),
        Dynamic(String),
    }

    type RouteFunction = dyn Fn(&Request, Response) -> Response + 'static;

    /// The Application struct is responsible for handling incoming requests and routing them to the appropriate handler function
    pub struct Application {
        static_methods: HashMap<(Method, String), Box<RouteFunction>>,
        dynamic_methods: Vec<(Method, Vec<RouteSegment>, Box<RouteFunction>)>,
    }

    impl Application {
        pub fn get<F>(&mut self, route: String, function: F)
        where
            F: Fn(&Request, Response) -> Response + 'static,
        {
            self.add_new_route(route, Method::GET, Box::new(function));
        }
        pub fn post<F>(&mut self, route: String, function: F)
        where
            F: Fn(&Request, Response) -> Response + 'static,
        {
            self.add_new_route(route, Method::POST, Box::new(function));
        }
        pub fn put<F>(&mut self, route: String, function: F)
        where
            F: Fn(&Request, Response) -> Response + 'static,
        {
            self.add_new_route(route, Method::PUT, Box::new(function));
        }
        pub fn patch<F>(&mut self, route: String, function: F)
        where
            F: Fn(&Request, Response) -> Response + 'static,
        {
            self.add_new_route(route, Method::PATCH, Box::new(function));
        }
        pub fn delete<F>(&mut self, route: String, function: F)
        where
            F: Fn(&Request, Response) -> Response + 'static,
        {
            self.add_new_route(route, Method::DELETE, Box::new(function));
        }
    }

    impl Application {
        // create a new application
        pub fn new() -> Application {
            return Application {
                static_methods: HashMap::new(),
                dynamic_methods: Vec::new(),
            };
        }

        // start the server , takes a port as argument
        pub fn listen(&mut self, port: i32) {
            let listener: TcpListener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();

            println!("Started server on port {}", port);

            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                let mut request = Request::new(&mut stream);

                self.execute_route(
                    request.route.to_string(),
                    request.method.clone(),
                    &mut request,
                    Response::new(),
                    &mut stream,
                );
            }
        }

        fn execute_route(
            &self,
            route: String,
            method: Method,
            request: &mut Request,
            response: Response,
            stream: &mut TcpStream,
        ) {
            let mut filtered_route = route;
            if filtered_route.contains('?') {
                let (route, query) = filtered_route.split_once('?').unwrap();
                let mut search_params_map = HashMap::new();
                for param in query.split('&') {
                    if let Some((name, value)) = param.split_once('=') {
                        search_params_map.insert(name.to_string(), value.to_string());
                    }
                }
                request.search_params = Some(search_params_map);
                filtered_route = route.to_string();
            }
            if filtered_route.starts_with("/") {
                if let Some(method) = self
                    .static_methods
                    .get(&(request.method.clone(), request.route.clone()))
                {
                    let f = method.as_ref();
                    f(&request, Response::new()).send(stream);
                } else {
                    let array: Vec<_> = filtered_route
                        .split('/')
                        .filter(|s| !s.is_empty())
                        .collect();
                    for (method_search, filtered_route, function) in self.dynamic_methods.iter() {
                        if array.len() != filtered_route.len() || method != *method_search {
                            continue;
                        }
                        let mut params_map = HashMap::new();
                        for (index, pattern) in array.iter().enumerate() {
                            match &filtered_route[index] {
                                RouteSegment::Static(s) => {
                                    if s != *pattern {
                                        continue;
                                    };
                                }
                                RouteSegment::Dynamic(s) => {
                                    params_map.insert(s.to_string(), pattern.to_string());
                                }
                            }
                        }
                        request.params = Some(params_map);
                        function(&request, response).send(stream);
                        return;
                    }
                }
            }
        }

        fn add_new_route(&mut self, path: String, method: Method, function: Box<RouteFunction>) {
            if path.contains(':') {
                let mut vec = Vec::new();
                let parts: Vec<_> = path
                    .trim()
                    .split('/')
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>();
                parts.iter().for_each(|item| {
                    if item.starts_with(':') {
                        vec.push(RouteSegment::Dynamic(item[1..].to_string()))
                    } else {
                        vec.push(RouteSegment::Static(item.to_string()));
                    }
                });
                self.dynamic_methods.push((method, vec, function));
            } else {
                self.static_methods.insert((method, path), function);
            }
        }
    }
}
