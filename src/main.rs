#![windows_subsystem = "windows"]
#![feature(extern_prelude)]
#![feature(plugin)]
#![plugin(phf_macros)]

extern crate web_view;

extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;

extern crate number_prefix;
extern crate reqwest;

extern crate phf;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;

extern crate regex;
extern crate semver;

extern crate zip;

mod assets;
mod config;
mod http;
mod installer;
mod rest;
mod sources;

use web_view::*;

use config::Config;

use installer::InstallerFramework;

use rest::WebServer;

// TODO: Fetch this over a HTTP request?
static RAW_CONFIG: &'static str = include_str!("../config.toml");

fn main() {
    let config = Config::from_toml_str(RAW_CONFIG).unwrap();

    let app_name = config.general.name.clone();

    let current_exe = std::env::current_exe().unwrap();
    let current_path = current_exe.parent().unwrap();
    let metadata_file = current_path.join("metadata.json");
    println!("Attempting to open: {:?}", metadata_file);
    let framework = if metadata_file.exists() {
        InstallerFramework::new_with_db(config, current_path).unwrap()
    } else {
        InstallerFramework::new(config)
    };

    // blah 1

    let server = WebServer::new(framework).unwrap();

    // Startup HTTP server for handling the web view
    let http_address = format!("http://{}", server.get_addr());
    println!("{}", http_address);

    // Init the web view
    let size = (1024, 550);
    let resizable = false;
    let debug = true;

    run(
        &format!("{} Installer", app_name),
        &http_address,
        Some(size),
        resizable,
        debug,
        |_| {},
        |_, _, _| {},
        (),
    );
}
