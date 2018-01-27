/// config.rs
///
/// Contains Config structures, as well as means of serialising them.

use toml;
use toml::de::Error as TomlError;

use serde_json::{self, Error as SerdeError};

/// Describes a overview of a individual package.
#[derive(Deserialize, Serialize, Clone)]
pub struct PackageDescription {
    name : String,
    description : String,
    default : Option<bool>
}

/// Describes the application itself.
#[derive(Deserialize, Serialize, Clone)]
pub struct GeneralConfig {
    name : String
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    general : GeneralConfig,
    packages : Vec<PackageDescription>
}

impl Config {
    /// Serialises as a JSON string.
    pub fn to_json_str(&self) -> Result<String, SerdeError> {
        serde_json::to_string(self)
    }

    /// Builds a configuration from a specified TOML string.
    pub fn from_toml_str(contents : &str) -> Result<Self, TomlError> {
        toml::from_str(contents)
    }
}
