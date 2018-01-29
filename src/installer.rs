/// installer.rs
///
/// Contains the main installer structure, as well as high-level means of controlling it.

use regex::Regex;

use std::env::home_dir;
use std::env::var;
use std::env::consts::OS;

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
        // TODO: Error handling
        println!("Framework: Installing {:?}", items);

        // Resolve items in config
        let mut to_install = Vec::new();

        for description in &self.config.packages {
            if items.contains(&description.name) {
                to_install.push(description.clone());
            }
        }

        println!("Resolved to {:?}", to_install);

        // Install packages
        for package in to_install.iter() {
            println!("Installing {}", package.name);

            let results = package.source.get_current_releases().unwrap();

            println!("Got releases");

            let filtered_regex = package.source.match_regex.replace("#PLATFORM#", OS);
            println!("Filtered regex: {}" , filtered_regex);
            let regex = Regex::new(&filtered_regex).unwrap();

            // Find the latest release in here
            let latest_result = results.into_iter()
                .filter(|f| f.files.iter().filter(|x| regex.is_match(x)).count() > 0)
                .max_by_key(|f| f.version.clone());
            println!("{:?}", latest_result);
        }
    }

    /// Creates a new instance of the Installer Framework with a specified Config.
    pub fn new(config: Config) -> Self {
        InstallerFramework { config }
    }
}
