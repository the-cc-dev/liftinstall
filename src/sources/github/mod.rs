/// github/mod.rs
use reqwest;
use reqwest::header::UserAgent;
use reqwest::StatusCode;

use serde_json;

use sources::types::*;

pub struct GithubReleases {}

/// The configuration for this release.
#[derive(Serialize, Deserialize)]
struct GithubConfig {
    repo: String,
}

impl GithubReleases {
    pub fn new() -> Self {
        GithubReleases {}
    }
}

impl ReleaseSource for GithubReleases {
    fn get_current_releases(&self, config: &TomlValue) -> Result<Vec<Release>, String> {
        // Reparse our Config as strongly typed
        let config : GithubConfig = match config.clone().try_into() {
            Ok(v) => v,
            Err(v) => return Err(format!("Failed to parse release config: {:?}", v)),
        };

        let mut results: Vec<Release> = Vec::new();

        // Build the HTTP client up
        let client = reqwest::Client::new();
        let mut response = client.get(&format!("https://api.github.com/repos/{}/releases", config.repo))
            .header(UserAgent::new("liftinstall (j-selby)"))
            .send()
            .map_err(|x| format!("Error while sending HTTP request: {:?}", x))?;

        if response.status() != StatusCode::Ok {
            return Err(format!("Bad status code: {:?}", response.status()));
        }

        let body = response.text()
            .map_err(|x| format!("Failed to decode HTTP response body: {:?}", x))?;

        let result: serde_json::Value = serde_json::from_str(&body)
            .map_err(|x| format!("Failed to parse response: {:?}", x))?;

        let result: &Vec<serde_json::Value> =
            result
                .as_array()
                .ok_or(format!("Response was not an array!"))?;

        // Parse JSON from server
        for entry in result.into_iter() {
            let mut files = Vec::new();

            let id: u64 = match entry["id"].as_u64() {
                Some(v) => v,
                None => return Err(format!("JSON payload missing information about ID")),
            };

            let assets = match entry["assets"].as_array() {
                Some(v) => v,
                None => return Err(format!("JSON payload not an array")),
            };

            for asset in assets.into_iter() {
                let string = match asset["name"].as_str() {
                    Some(v) => v,
                    None => {
                        return Err(format!(
                            "JSON payload missing information about release name"
                        ))
                    }
                };

                let url = match asset["browser_download_url"].as_str() {
                    Some(v) => v,
                    None => {
                        return Err(format!(
                            "JSON payload missing information about release URL"
                        ))
                    }
                };

                files.push(File {
                    name: string.to_string(),
                    url: url.to_string(),
                });
            }

            results.push(Release {
                version: Version::new_number(id),
                files,
            });
        }

        Ok(results)
    }
}
