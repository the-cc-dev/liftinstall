//! main.rs
//!
//! The main entrypoint for the application. Orchestrates the building of the installation
//! framework, and opens necessary HTTP servers/frontends.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![deny(unsafe_code)]
#![deny(missing_docs)]

#[cfg(windows)]
extern crate nfd;

extern crate web_view;

extern crate futures;
extern crate hyper;
extern crate url;

extern crate number_prefix;
extern crate reqwest;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;

extern crate regex;
extern crate semver;

extern crate dirs;
extern crate zip;

extern crate fern;
#[macro_use]
extern crate log;

extern crate chrono;

extern crate clap;

mod assets;
mod config;
mod http;
mod installer;
mod logging;
mod rest;
mod sources;
mod tasks;

use web_view::*;

use installer::InstallerFramework;

#[cfg(windows)]
use nfd::Response;

use rest::WebServer;

use std::net::ToSocketAddrs;

use std::net::TcpListener;
use std::sync::Arc;
use std::sync::RwLock;

use logging::LoggingErrors;

use clap::App;
use clap::Arg;
use log::Level;

use config::BaseAttributes;

static RAW_CONFIG: &'static str = include_str!("../config.toml");

#[derive(Deserialize, Debug)]
enum CallbackType {
    SelectInstallDir { callback_name: String },
    Log { msg: String, kind: String },
}

fn main() {
    logging::setup_logger().expect("Unable to setup logging!");

    let config =
        BaseAttributes::from_toml_str(RAW_CONFIG).log_expect("Config file could not be read");

    let app_name = config.name.clone();

    let matches = App::new(format!("{} installer", app_name))
        .version(env!("CARGO_PKG_VERSION"))
        .about(format!("An interactive installer for {}", app_name).as_ref())
        .arg(
            Arg::with_name("launcher")
                .long("launcher")
                .value_name("TARGET")
                .help("Launches the specified executable after checking for updates")
                .takes_value(true),
        )
        .get_matches();

    info!("{} installer", app_name);

    let current_exe = std::env::current_exe().log_expect("Current executable could not be found");
    let current_path = current_exe
        .parent()
        .log_expect("Parent directory of executable could not be found");
    let metadata_file = current_path.join("metadata.json");
    let mut framework = if metadata_file.exists() {
        info!("Using pre-existing metadata file: {:?}", metadata_file);
        InstallerFramework::new_with_db(config, current_path).log_expect("Unable to parse metadata")
    } else {
        info!("Starting fresh install");
        InstallerFramework::new(config)
    };

    let is_launcher = if let Some(string) = matches.value_of("launcher") {
        framework.is_launcher = true;
        framework.launcher_path = Some(string.to_string());
        true
    } else {
        false
    };

    // Firstly, allocate us an epidermal port
    let target_port = {
        let listener = TcpListener::bind("127.0.0.1:0")
            .log_expect("At least one local address should be free");
        listener
            .local_addr()
            .log_expect("Should be able to pull address from listener")
            .port()
    };

    // Now, iterate over all ports
    let addresses = "localhost:0"
        .to_socket_addrs()
        .log_expect("No localhost address found");

    let mut servers = Vec::new();
    let mut http_address = None;

    let framework = Arc::new(RwLock::new(framework));

    // Startup HTTP server for handling the web view
    for mut address in addresses {
        address.set_port(target_port);

        let server = WebServer::with_addr(framework.clone(), address)
            .log_expect("Failed to bind to address");

        info!("Server: {:?}", address);

        http_address = Some(address);

        servers.push(server);
    }

    let http_address = http_address.log_expect("No HTTP address found");

    let http_address = format!("http://localhost:{}", http_address.port());

    // Init the web view
    let size = if is_launcher { (600, 300) } else { (1024, 500) };

    let resizable = false;
    let debug = true;

    run(
        &format!("{} Installer", app_name),
        Content::Url(http_address),
        Some(size),
        resizable,
        debug,
        |_| {},
        |wv, msg, _| {
            let command: CallbackType =
                serde_json::from_str(msg).log_expect(&format!("Unable to parse string: {:?}", msg));

            debug!("Incoming payload: {:?}", command);

            match command {
                CallbackType::SelectInstallDir { callback_name } => {
                    #[cfg(windows)]
                    let result = match nfd::open_pick_folder(None)
                        .log_expect("Unable to open folder dialog")
                    {
                        Response::Okay(v) => v,
                        _ => return,
                    };

                    #[cfg(not(windows))]
                    let result =
                        wv.dialog(Dialog::ChooseDirectory, "Select a install directory...", "");

                    if !result.is_empty() {
                        let result = serde_json::to_string(&result)
                            .log_expect("Unable to serialize response");
                        let command = format!("{}({});", callback_name, result);
                        debug!("Injecting response: {}", command);
                        wv.eval(&command);
                    }
                }
                CallbackType::Log { msg, kind } => {
                    let kind = match kind.as_ref() {
                        "info" | "log" => Level::Info,
                        "warn" => Level::Warn,
                        "error" => Level::Error,
                        _ => Level::Error,
                    };

                    log!(target: "liftinstall::frontend-js", kind, "{}", msg);
                }
            }
        },
        (),
    );
}
