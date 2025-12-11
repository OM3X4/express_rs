
mod request {
    use std::collections::HashMap;


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
}

mod express {
    struct Application;

    trait Server {
        fn run(port: i32);
    }
}
