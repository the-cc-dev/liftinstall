/// config.rs
///
/// Contains Config structures, as well as means of serialising them.
use toml;
use toml::de::Error as TomlError;

use serde_json::{self, Error as SerdeError};

use sources::get_by_name;
use sources::types::Release;

/// Description of the source of a package.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PackageSource {
    pub name: String,
    #[serde(rename = "match")]
    pub match_regex: String,
    pub config: toml::Value,
}

/// Describes a overview of a individual package.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PackageDescription {
    pub name: String,
    pub description: String,
    pub default: Option<bool>,
    pub source: PackageSource,
}

/// Describes the application itself.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GeneralConfig {
    pub name: String,
    pub installing_message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub general: GeneralConfig,
    pub packages: Vec<PackageDescription>,
}

impl Config {
    /// Serialises as a JSON string.
    pub fn to_json_str(&self) -> Result<String, SerdeError> {
        serde_json::to_string(self)
    }

    /// Builds a configuration from a specified TOML string.
    pub fn from_toml_str(contents: &str) -> Result<Self, TomlError> {
        toml::from_str(contents)
    }
}

impl PackageSource {
    /// Fetches releases for a given package
    pub fn get_current_releases(&self) -> Result<Vec<Release>, String> {
        let package_handler = match get_by_name(&self.name) {
            Some(v) => v,
            _ => return Err(format!("Handler {} not found", self.name)),
        };

        package_handler.get_current_releases(&self.config)
    }
}
