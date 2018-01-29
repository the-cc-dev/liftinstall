/// config.rs
///
/// Contains Config structures, as well as means of serialising them.

use toml;
use toml::de::Error as TomlError;

use serde_json::{self, Error as SerdeError};

/// Description of the source of a package.
#[derive(Deserialize, Serialize, Clone)]
pub struct PackageSource {
    pub name : String,
    pub config : toml::Value
}

/// Describes a overview of a individual package.
#[derive(Deserialize, Serialize, Clone)]
pub struct PackageDescription {
    pub name: String,
    pub description: String,
    pub default: Option<bool>,
    pub source: PackageSource
}

/// Describes the application itself.
#[derive(Deserialize, Serialize, Clone)]
pub struct GeneralConfig {
    pub name: String,
    pub installing_message: String,
}

#[derive(Deserialize, Serialize, Clone)]
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
