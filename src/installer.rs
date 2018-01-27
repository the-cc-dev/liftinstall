/// installer.rs
///
/// Contains the main installer structure, as well as high-level means of controlling it.

use config::Config;

/// The installer framework contains metadata about packages, what is installable, what isn't,
/// etc.
pub struct InstallerFramework {
    config : Config
}

impl InstallerFramework {
    /// Returns a copy of the configuration.
    pub fn get_config(&self) -> Config {
        self.config.clone()
    }

    /// Creates a new instance of the Installer Framework with a specified Config.
    pub fn new(config : Config) -> Self {
        InstallerFramework {
            config
        }
    }
}
