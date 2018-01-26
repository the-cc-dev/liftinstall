//#![windows_subsystem = "windows"]

extern crate web_view;
extern crate tiny_http;

extern crate includedir;
extern crate phf;

mod assets;

use web_view::*;

use tiny_http::Server;
use tiny_http::Response;
use tiny_http::Header;

use std::thread;
use std::str::FromStr;

fn main() {
    // Startup HTTP server for handling the web view
    let server = Server::http("127.0.0.1:0").unwrap();

    let http_address = format!("http://{}", server.server_addr());
    println!("{}", format!("{}", server.server_addr()));

    let _ = thread::spawn(move || {
        loop {
            // blocks until the next request is received
            let request = match server.recv() {
                Ok(rq) => rq,
                Err(e) => { println!("error: {}", e); break }
            };

            // Work out what they want
            let mut url : String = request.url().into();
            if url.ends_with("/") {
                url += "index.html";
            }

            println!("Requesting: {}", url);

            match assets::file_from_string(&url) {
                Some((content_type, file)) => {
                    let mut response = Response::from_data(file);
                    if let Some(content_type) = content_type {
                        response.add_header(Header::from_str(
                            &format!("Content-Type:{}", content_type)).unwrap())
                    }

                    request.respond(response)
                },
                None => request.respond(Response::empty(404))
            }.unwrap();
        }
    });

    // Init the web view
    let size = (1024, 550);
    let resizable = false;
    let debug = true;
    let init_cb = |_| {};
    let userdata = ();

    run(
        "yuzu Installer",
        &http_address,
        Some(size),
        resizable,
        debug,
        init_cb,
        /* frontend_cb: */ |_, _, _| {},
        userdata
    );
}
