
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

    let mut server =  express::Application::new();

    server.get(String::from("/hello"), |req , _res| {

        println!("The Request is \n{:#?}" , req);


        return _res.status(201).json(String::from(r#"{"name":"omar"}"#));
    });
    server.get(String::from("/omar") , |req , _res| {
        println!("The Request is \n{:#?}" , req);

        let html = r##"
        <!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <title>Hello!</title>
  </head>
  <body>
    <h1>Omar Emad!</h1>
    <p>This is Omar.</p>
  </body>
</html>
        "##;

        return _res.status(200).html(html.to_string());
    });

    server.get("/omar/:id/:name".to_string(), |req, res| {

        println!("id is {}" , req.params.as_ref().unwrap().get("id").unwrap());
        println!("name is {}" , req.params.as_ref().unwrap().get("name").unwrap());

        res
    });

    server.listen(7878)

}
