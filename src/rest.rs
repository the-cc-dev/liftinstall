/// rest.rs
///
/// Provides a HTTP/REST server for both frontend<->backend communication, as well
/// as talking to external applications.

use tiny_http::{Server, Request, Response, Header};

use nfd;
use nfd::Response as NfdResponse;

use serde_json;

use std::error::Error;
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::thread::{self, JoinHandle};
use std::str::FromStr;

use assets;

use installer::InstallerFramework;

#[derive(Serialize)]
struct FileSelection {
    path : Option<String>
}

/// Encapsulates tiny_http's state.
pub struct WebServer {
    server : Server,
    framework : InstallerFramework
}

impl WebServer {
    /// Handles a Web Server request.
    fn handle_request(&self, request : Request) {
        // Work out what they want
        let mut url : String = request.url().into();

        println!("Serving request: {}", url);

        // Capture API calls before they fall into phf
        if url.starts_with("/api/") {
            let api_url = &url[5 ..];
            let call_response = self.rest_call(api_url);

            match call_response {
                Some(response) => request.respond(Response::from_data(response.into_bytes())
                    .with_header(Header::from_str("Content-Type: application/json")
                        .unwrap())),
                None => request.respond(Response::empty(404))
            }.unwrap();
            return;
        }

        // At this point, we have a web browser client. Search for a index page
        // if needed
        if url.ends_with("/") {
            url += "index.html";
        }

        match assets::file_from_string(&url) {
            Some((content_type, file)) => {
                let mut response = Response::from_data(file);
                if let Some(content_type) = content_type {
                    response.add_header(Header::from_str(
                        &format!("Content-Type: {}", content_type)).unwrap())
                }

                request.respond(response)
            },
            None => request.respond(Response::empty(404))
        }.unwrap();
    }

    /// Makes a call to a REST endpoint.
    fn rest_call(&self, path : &str) -> Option<String> {
        let mut path = path.to_owned();

        // Strip off query params
        let query = match path.rfind("?") {
            Some(pos) => {
                let ext = path[pos + 1 ..].to_owned();
                path = path[0 .. pos].to_owned();

                Some(ext)
            },
            None => None
        };

        match path.as_str() {
            // This endpoint should be usable directly from a <script> tag during loading.
            // TODO: Handle errors
            "config" => Some(enscapsulate_json("config",
                                          &self.framework.get_config().to_json_str().unwrap())),
            "file-select" => {
                let file_dialog = nfd::open_pick_folder(None).unwrap();
                let file = match file_dialog {
                    NfdResponse::Okay(path) => Some(path),
                    _ => None
                };

                let response = FileSelection {
                    path : file
                };

                Some(serde_json::to_string(&response).unwrap())
            },
            "default-path" => {
                let path = self.framework.get_default_path();

                let response = FileSelection {
                    path
                };

                Some(serde_json::to_string(&response).unwrap())
            },
            _ => None
        }
    }

    /// Runs the main loop of the web server.
    fn run(&mut self) {
        loop {
            // Take a single request from the client
            let request = match self.server.recv() {
                Ok(rq) => rq,
                Err(e) => { println!("error: {}", e); break }
            };

            self.handle_request(request);
        }
    }

    /// Starts the webserver. Consumes the entity.
    pub fn start(mut self) -> JoinHandle<()> {
        thread::spawn(move || {
            self.run()
        })
    }

    /// Returns the bound address that the server is running from.
    pub fn get_addr(&self) -> SocketAddr {
        self.server.server_addr()
    }

    /// Creates a new web server, bound to a random port on localhost.
    pub fn new(framework : InstallerFramework) -> Result<Self, Box<Error + Send + Sync + 'static>> {
        WebServer::with_addr(framework, SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
    }

    /// Creates a new web server with the specified address.
    pub fn with_addr(framework : InstallerFramework, addr : SocketAddr)
        -> Result<Self, Box<Error + Send + Sync + 'static>> {
        let server = Server::http(addr)?;

        Ok(WebServer {
            server,
            framework
        })
    }
}

/// Encapsulates JSON as a injectable Javascript script.
fn enscapsulate_json(field_name : &str, json : &str) -> String {
    format!("var {} = {};", field_name, json)
}