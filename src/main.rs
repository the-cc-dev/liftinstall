#![windows_subsystem = "windows"]

extern crate web_view;

extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;

extern crate includedir;
extern crate phf;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;

extern crate semver;

mod assets;
mod rest;
mod config;
mod installer;
mod sources;

use web_view::*;

use config::Config;

use installer::InstallerFramework;

use rest::WebServer;

// TODO: Fetch this over a HTTP request?
static RAW_CONFIG: &'static str = include_str!("../config.toml");

fn main() {
    let config = Config::from_toml_str(RAW_CONFIG).unwrap();

    /*let github_releases = sources::github::GithubReleases{};
    println!("{:?}", github_releases.get_current_releases(&config.packages[0].source.config));*/

    let app_name = config.general.name.clone();

    let framework = InstallerFramework::new(config);

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
