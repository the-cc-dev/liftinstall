/// rest.rs
///
/// Provides a HTTP/REST server for both frontend<->backend communication, as well
/// as talking to external applications.
extern crate nfd;
extern crate url;

use self::nfd::Response as NfdResponse;

use serde_json;

use futures::Stream;
use futures::Future;
use futures::future;
use futures::Sink;

use hyper::{self, Error as HyperError, Get, Post, StatusCode};
use hyper::header::{ContentLength, ContentType};
use hyper::server::{Http, Request, Response, Service};

use self::url::form_urlencoded;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::thread::{self, JoinHandle};
use std::process::exit;
use std::sync::Arc;
use std::sync::mpsc::channel;
use std::collections::HashMap;

use assets;

use installer::InstallerFramework;
use installer::InstallMessage;

#[derive(Serialize)]
struct FileSelection {
    path: Option<String>,
}

/// Acts as a communication mechanism between the Hyper WebService and the rest of the
/// application.
pub struct WebServer {
    _handle: JoinHandle<()>,
    addr: SocketAddr,
}

impl WebServer {
    /// Returns the bound address that the server is running from.
    pub fn get_addr(&self) -> SocketAddr {
        self.addr.clone()
    }

    /// Creates a new web server, bound to a random port on localhost.
    pub fn new(framework: InstallerFramework) -> Result<Self, HyperError> {
        WebServer::with_addr(
            framework,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0),
        )
    }

    /// Creates a new web server with the specified address.
    pub fn with_addr(framework: InstallerFramework, addr: SocketAddr) -> Result<Self, HyperError> {
        let (sender, receiver) = channel();

        let handle = thread::spawn(move || {
            let shared_framework = Arc::new(framework);

            let server = Http::new()
                .bind(&addr, move || {
                    Ok(WebService {
                        framework: shared_framework.clone(),
                    })
                })
                .unwrap();

            sender.send(server.local_addr().unwrap()).unwrap();

            server.run().unwrap();
        });

        let addr = receiver.recv().unwrap();

        Ok(WebServer {
            _handle: handle,
            addr,
        })
    }
}

/// Holds internal state for Hyper
struct WebService {
    framework: Arc<InstallerFramework>,
}

impl Service for WebService {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    /// HTTP request handler
    fn call(&self, req: Self::Request) -> Self::Future {
        Box::new(future::ok(match (req.method(), req.path()) {
            // This endpoint should be usable directly from a <script> tag during loading.
            // TODO: Handle errors
            (&Get, "/api/config") => {
                let file = enscapsulate_json(
                    "config",
                    &self.framework.get_config().to_json_str().unwrap(),
                );

                Response::<hyper::Body>::new()
                    .with_header(ContentLength(file.len() as u64))
                    .with_header(ContentType::json())
                    .with_body(file)
            }
            // Opens a file dialog and returns the path as a string
            (&Get, "/api/file-select") => {
                let file_dialog = nfd::open_pick_folder(None).unwrap();
                let file = match file_dialog {
                    NfdResponse::Okay(path) => Some(path),
                    _ => None,
                };

                let response = FileSelection { path: file };

                let file = serde_json::to_string(&response).unwrap();

                Response::<hyper::Body>::new()
                    .with_header(ContentLength(file.len() as u64))
                    .with_header(ContentType::json())
                    .with_body(file)
            }
            // Returns the default path for a installation
            (&Get, "/api/default-path") => {
                let path = self.framework.get_default_path();

                let response = FileSelection { path };

                let file = serde_json::to_string(&response).unwrap();

                Response::<hyper::Body>::new()
                    .with_header(ContentLength(file.len() as u64))
                    .with_header(ContentType::json())
                    .with_body(file)
            }
            // Immediately exits the application
            (&Get, "/api/exit") => {
                exit(0);
            }
            (&Post, "/api/start-install") => {
                // We need to bit of pipelining to get this to work
                let cloned_element = self.framework.clone();

                return Box::new(req.body().concat2().map(move |b| {
                    let results = form_urlencoded::parse(b.as_ref())
                        .into_owned()
                        .collect::<HashMap<String, String>>();

                    let mut to_install = Vec::new();

                    // Transform results into just an array of stuff to install
                    for (key, value) in results.iter() {
                        if value == "true" {
                            to_install.push(key.to_owned());
                        }
                    }

                    let (sender, receiver) = channel();
                    let (tx, rx) = hyper::Body::pair();

                    // Startup a thread to do this operation for us
                    thread::spawn(move || {
                        match cloned_element.install(to_install, &sender) {
                            Err(v) => sender.send(InstallMessage::Error(v)).unwrap(),
                            _ => {}
                        }
                        sender.send(InstallMessage::EOF).unwrap();
                    });

                    // Spawn a thread for transforming messages to chunk messages
                    thread::spawn(move || {
                        let mut tx = tx;
                        loop {
                            let response = receiver.recv().unwrap();

                            match &response {
                                &InstallMessage::EOF => break,
                                _ => {}
                            }

                            let mut response = serde_json::to_string(&response).unwrap();
                            response.push('\n');
                            tx = tx.send(Ok(response.into_bytes().into())).wait().unwrap();
                        }
                    });

                    Response::<hyper::Body>::new()
                        //.with_header(ContentLength(file.len() as u64))
                        .with_header(ContentType::plaintext())
                        .with_body(rx)
                }));
            }

            // Static file handler
            (&Get, _) => {
                // At this point, we have a web browser client. Search for a index page
                // if needed
                let mut path: String = req.path().to_owned();
                if path.ends_with("/") {
                    path += "index.html";
                }

                match assets::file_from_string(&path) {
                    Some((content_type, file)) => {
                        let content_type = ContentType(content_type.parse().unwrap());
                        Response::<hyper::Body>::new()
                            .with_header(ContentLength(file.len() as u64))
                            .with_header(content_type)
                            .with_body(file)
                    }
                    None => Response::new().with_status(StatusCode::NotFound),
                }
            }
            // Fallthrough for POST/PUT/CONNECT/...
            _ => Response::new().with_status(StatusCode::NotFound),
        }))
    }
}

/// Encapsulates JSON as a injectable Javascript script.
fn enscapsulate_json(field_name: &str, json: &str) -> String {
    format!("var {} = {};", field_name, json)
}
