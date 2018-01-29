/// installer.rs
///
/// Contains the main installer structure, as well as high-level means of controlling it.

use std::env::home_dir;
use std::env::var;

use std::path::PathBuf;

use config::Config;

/// The installer framework contains metadata about packages, what is installable, what isn't,
/// etc.
pub struct InstallerFramework {
    config: Config,
}

impl InstallerFramework {
    /// Returns a copy of the configuration.
    pub fn get_config(&self) -> Config {
        self.config.clone()
    }

    /// Returns the default install path.
    pub fn get_default_path(&self) -> Option<String> {
        let app_name = &self.config.general.name;

        let base_dir = match var("LOCALAPPDATA") {
            Ok(path) => PathBuf::from(path),
            Err(_) => home_dir()?,
        };

        let file = base_dir.join(app_name);

        Some(file.to_str()?.to_owned())
    }

    /// Sends a request for something to be installed.
    pub fn install(&self, items : Vec<String>) {
        println!("Framework: Installing {:?}", items);
    }

    /// Creates a new instance of the Installer Framework with a specified Config.
    pub fn new(config: Config) -> Self {
        InstallerFramework { config }
    }
}
