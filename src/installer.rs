/// installer.rs
///
/// Contains the main installer structure, as well as high-level means of controlling it.

use regex::Regex;

use std::fs::create_dir_all;
use std::fs::read_dir;

use std::env::home_dir;
use std::env::var;
use std::env::consts::OS;

use std::path::PathBuf;

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::Sender;

use config::Config;

use http::stream_file;

use number_prefix::{decimal_prefix, Prefixed, Standalone};

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

struct DownloadProgress {
    downloaded: usize,
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
        path: &str,
        messages: &Sender<InstallMessage>,
    ) -> Result<(), String> {
        // TODO: Error handling
        println!("Framework: Installing {:?} to {}", items, path);

        // Create our install directory
        let path = PathBuf::from(path);
        if !path.exists() {
            match create_dir_all(&path) {
                Ok(_) => {},
                Err(v) => return Err(format!("Failed to create install directory: {:?}", v)),
            }
        }

        if !path.is_dir() {
            return Err(format!("Install destination is not a directory."));
        }

        // Make sure it is empty
        let paths = match read_dir(path) {
            Ok(v) => v,
            Err(v) => return Err(format!("Failed to read install destination: {:?}", v)),
        };

        if paths.count() != 0 {
            return Err(format!("Install destination is not empty."));
        }

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

            // Download this file
            let lock = Arc::new(Mutex::new(DownloadProgress { downloaded: 0 }));

            stream_file(latest_file.url, |data, size| {
                let mut reference = lock.lock().unwrap();
                reference.downloaded += data.len();

                let base_percentage = base_package_percentage + base_package_range * 0.50;
                let range_percentage = base_package_range / 2.0;

                let global_percentage = if size == 0 {
                    base_percentage
                } else {
                    let download_percentage = (reference.downloaded as f64) / (size as f64);
                    // Split up the bar for this download in half (for metadata download + parse), then
                    // add on our current percentage
                    base_percentage + range_percentage * download_percentage
                };

                // Pretty print data volumes
                let pretty_current = match decimal_prefix(reference.downloaded as f64) {
                    Standalone(bytes) => format!("{} bytes", bytes),
                    Prefixed(prefix, n) => format!("{:.0} {}B", n, prefix),
                };
                let pretty_total = match decimal_prefix(size as f64) {
                    Standalone(bytes) => format!("{} bytes", bytes),
                    Prefixed(prefix, n) => format!("{:.0} {}B", n, prefix),
                };

                messages
                    .send(InstallMessage::Status(
                        format!(
                            "Downloading {} ({} of {})",
                            package.name, pretty_current, pretty_total
                        ),
                        global_percentage,
                    ))
                    .unwrap();
            })?;

            println!("File downloaded successfully");

            count += 1.0;
        }

        Ok(())
    }

    /// Creates a new instance of the Installer Framework with a specified Config.
    pub fn new(config: Config) -> Self {
        InstallerFramework { config }
    }
}
