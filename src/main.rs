
fn main() {
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

        println!("id is {}" , req.get_param("id").unwrap());
        println!("name is {}" , req.get_param("name").unwrap());

        println!("id is {}" , req.get_search_param("id").unwrap_or("undefined".to_string()));
        println!("name is {}" , req.get_search_param("name").unwrap_or("undefined".to_string()));



        res
    });

    server.listen(7878)

}
