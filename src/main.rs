#![windows_subsystem = "windows"]

extern crate web_view;
extern crate tiny_http;

extern crate includedir;
extern crate phf;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;

mod assets;
mod rest;
mod config;
mod installer;

use web_view::*;

use config::Config;

use installer::InstallerFramework;

use rest::WebServer;

// TODO: Fetch this over a HTTP request?
static RAW_CONFIG : &'static str = include_str!("../config.toml");

fn main() {
    let config = Config::from_toml_str(RAW_CONFIG).unwrap();

    let framework = InstallerFramework::new(config);

    let server = WebServer::new(framework).unwrap();

    // Startup HTTP server for handling the web view
    let http_address = format!("http://{}", server.get_addr());
    println!("{}", http_address);

    server.start();

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
