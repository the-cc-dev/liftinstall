/// github/mod.rs

use futures::{Future, Stream};

use tokio_core::reactor::Core;

use hyper::Client;
use hyper::Uri;
use hyper::Method;
use hyper::Request;
use hyper::header::UserAgent;

use hyper_tls::HttpsConnector;

use toml;

use serde_json;

use sources::types::*;

pub struct GithubReleases {
}

/// The configuration for this release.
#[derive(Serialize, Deserialize)]
struct GithubConfig {
    repo : String
}

impl ReleaseSource for GithubReleases {
    fn get_current_releases(&self, config: &TomlValue) -> Result<Vec<Release>, String> {
        // Reparse our Config as strongly typed
        let config_string = match toml::to_string(config) {
            Ok(v) => v,
            Err(v) => return Err(format!("Failed to convert config: {:?}", v))
        };

        let config : GithubConfig = match toml::from_str(&config_string) {
            Ok(v) => v,
            Err(v) => return Err(format!("Failed to convert config: {:?}", v))
        };

        let mut core = match Core::new() {
            Ok(v) => v,
            Err(v) => return Err(format!("Failed to init Tokio: {:?}", v))
        };

        // Build the HTTP client up
        let client = Client::configure()
            .connector(match HttpsConnector::new(4, &core.handle()) {
                Ok(v) => v,
                Err(v) => return Err(format!("Failed to init https: {:?}", v))
            })
            .build(&core.handle());

        let mut results: Vec<Release> = Vec::new();
        let target_url : Uri = match format!("https://api.github.com/repos/{}/releases",
                                             config.repo).parse() {
            Ok(v) => v,
            Err(v) => return Err(format!("Failed to generate target url: {:?}", v))
        };

        let mut req = Request::new(Method::Get, target_url);
        req.headers_mut().set(UserAgent::new("installer-rs (j-selby)"));

        // Build our future
        let future = client.request(req).and_then(|res| {
            res.body().concat2().and_then(move |body| {
                let raw_json : Result<serde_json::Value, String>
                    = match serde_json::from_slice(&body) {
                    Ok(v) => Ok(v),
                    Err(v) => Err(format!("Failed to parse response: {:?}", v))
                };

                Ok(raw_json)
            })
        });

        // Unwrap the future's results
        let result : serde_json::Value = match core.run(future) {
            Ok(v) => v,
            Err(v) => return Err(format!("Failed to fetch info: {:?}", v))
        }?;

        let result : &Vec<serde_json::Value>  = match result.as_array() {
            Some(v) => v,
            None => return Err(format!("JSON payload not an array"))
        };

        // Parse JSON from server
        for entry in result.into_iter() {
            let mut files = Vec::new();

            let id : u64 = match entry["id"].as_u64() {
                Some(v) => v,
                None => return Err(format!("JSON payload missing information about ID"))
            };

            let assets = match entry["assets"].as_array() {
                Some(v) => v,
                None => return Err(format!("JSON payload not an array"))
            };

            for asset in assets.into_iter() {
                let string = match asset["name"].as_str() {
                    Some(v) => v,
                    None => return Err(format!("JSON payload missing information about ID"))
                };

                files.push(string.to_owned());
            }

            results.push(Release {
                version: Version::new_number(id),
                files
            });
        }

        Ok(results)
    }
}
