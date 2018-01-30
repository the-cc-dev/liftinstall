/// installer.rs
///
/// Contains the main installer structure, as well as high-level means of controlling it.

use regex::Regex;

use std::env::home_dir;
use std::env::var;
use std::env::consts::OS;

use std::path::PathBuf;

use std::sync::mpsc::Sender;

use config::Config;

/// A message thrown during the installation of packages.
#[derive(Serialize)]
pub enum InstallMessage {
    Status(String, f64),
    Error(String),
    EOF,
}

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
    pub fn install(
        &self,
        items: Vec<String>,
        messages: &Sender<InstallMessage>,
    ) -> Result<(), String> {
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
        let mut count = 0.0 as f64;
        let max = to_install.len() as f64;

        for package in to_install.iter() {
            let base_package_percentage = count / max;
            let base_package_range = ((count + 1.0) / max) - base_package_percentage;

            println!("Installing {}", package.name);

            messages
                .send(InstallMessage::Status(
                    format!(
                        "Polling {} for latest version of {}",
                        package.source.name, package.name
                    ),
                    base_package_percentage + base_package_range * 0.25,
                ))
                .unwrap();

            let results = package.source.get_current_releases()?;

            messages
                .send(InstallMessage::Status(
                    format!("Resolving dependency for {}", package.name),
                    base_package_percentage + base_package_range * 0.50,
                ))
                .unwrap();

            let filtered_regex = package.source.match_regex.replace("#PLATFORM#", OS);
            let regex = match Regex::new(&filtered_regex) {
                Ok(v) => v,
                Err(v) => return Err(format!("An error occured while compiling regex: {:?}", v)),
            };

            // Find the latest release in here
            let latest_result = results
                .into_iter()
                .filter(|f| f.files.iter().filter(|x| regex.is_match(&x.name)).count() > 0)
                .max_by_key(|f| f.version.clone());

            let latest_result = match latest_result {
                Some(v) => v,
                None => return Err(format!("No release with correct file found")),
            };

            // Find the matching file in here
            let latest_file = latest_result
                .files
                .into_iter()
                .filter(|x| regex.is_match(&x.name))
                .next()
                .unwrap();

            println!("{:?}", latest_file);

            // TODO: Download found file

            count += 1.0;
        }

        Ok(())
    }

    /// Creates a new instance of the Installer Framework with a specified Config.
    pub fn new(config: Config) -> Self {
        InstallerFramework { config }
    }
}
