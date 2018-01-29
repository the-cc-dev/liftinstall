/// github/mod.rs

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
        unimplemented!()
    }
}
