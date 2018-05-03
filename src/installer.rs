/// installer.rs
///
/// Contains the main installer structure, as well as high-level means of controlling it.

use regex::Regex;

use zip::ZipArchive;

use serde_json;

use number_prefix::{decimal_prefix, Prefixed, Standalone};

use std::fs::create_dir_all;
use std::fs::read_dir;
use std::fs::File;

use std::env::home_dir;
use std::env::var;
use std::env::current_exe;
use std::env::consts::OS;

use std::path::Path;
use std::path::PathBuf;

use std::io::Cursor;
use std::io::copy;

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::Sender;

use config::Config;

use http::stream_file;

use sources::types::Version;

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
    database: Vec<LocalInstallation>,
    install_path: Option<String>,
    preexisting_install: bool
}

/// Used to track the amount of data that has been downloaded during a HTTP request.
struct DownloadProgress {
    downloaded: usize,
}

/// Tracks the state of a local installation
#[derive(Debug, Serialize, Deserialize)]
struct LocalInstallation {
    name: String,
    version: Version,
    files: Vec<String>,
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
    /// items: Array of named packages to be installed
    /// messages: Channel used to send progress messages
    /// fresh_install: If the install directory must be empty
    pub fn install(
        &mut self,
        items: Vec<String>,
        messages: &Sender<InstallMessage>,
        fresh_install: bool
    ) -> Result<(), String> {
        // We have to have a install path for us to be able to do anything
        let path = match self.install_path.clone() {
            Some(v) => v,
            None => return Err(format!("No install directory for installer"))
        };

        println!("Framework: Installing {:?} to {}", items, path);

        // Create our install directory
        let path = PathBuf::from(path);
        if !path.exists() {
            match create_dir_all(&path) {
                Ok(_) => {}
                Err(v) => return Err(format!("Failed to create install directory: {:?}", v)),
            }
        }

        if !path.is_dir() {
            return Err(format!("Install destination is not a directory."));
        }

        // Make sure it is empty
        if fresh_install {
            let paths = match read_dir(&path) {
                Ok(v) => v,
                Err(v) => return Err(format!("Failed to read install destination: {:?}", v)),
            };

            if paths.count() != 0 {
                return Err(format!("Install destination is not empty."));
            }
        }

        // Resolve items in config
        let mut to_install = Vec::new();

        for description in &self.config.packages {
            if items.contains(&description.name) {
                to_install.push(description.clone());
            }
        }

        println!("Resolved to {:?}", to_install);

        // TODO: Uninstall pre-existing packages

        // Install packages
        let mut count = 0.0 as f64;
        let max = to_install.len() as f64;

        for package in to_install.iter() {
            let base_package_percentage = count / max;
            let base_package_range = ((count + 1.0) / max) - base_package_percentage;

            println!("Installing {}", package.name);

            // 10%: polling
            messages
                .send(InstallMessage::Status(
                    format!(
                        "Polling {} for latest version of {}",
                        package.source.name, package.name
                    ),
                    base_package_percentage + base_package_range * 0.10,
                ))
                .unwrap();

            let results = package.source.get_current_releases()?;

            // 20%: waiting for parse/HTTP
            messages
                .send(InstallMessage::Status(
                    format!("Resolving dependency for {}", package.name),
                    base_package_percentage + base_package_range * 0.20,
                ))
                .unwrap();

            let filtered_regex = package.source.match_regex.replace("#PLATFORM#", OS);
            let regex = match Regex::new(&filtered_regex) {
                Ok(v) => v,
                Err(v) => return Err(format!("An error occured while compiling regex: {:?}", v)),
            };

            println!("Releases: {:?}", results);

            // Find the latest release in here
            let latest_result = results
                .into_iter()
                .filter(|f| f.files.iter().filter(|x| regex.is_match(&x.name)).count() > 0)
                .max_by_key(|f| f.version.clone());

            let latest_result = match latest_result {
                Some(v) => v,
                None => return Err(format!("No release with correct file found")),
            };

            let latest_version = latest_result.version.clone();

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
            let data_storage: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));

            // 33-66%: downloading file
            stream_file(latest_file.url, |data, size| {
                {
                    let mut data_lock = data_storage.lock().unwrap();
                    data_lock.extend_from_slice(&data);
                }

                let mut reference = lock.lock().unwrap();
                reference.downloaded += data.len();

                let base_percentage = base_package_percentage + base_package_range * 0.33;
                let range_percentage = base_package_range / 3.0;

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

            // Extract this downloaded file
            let mut installed_files = Vec::new();

            // TODO: Handle files other then zips
            // TODO: Make database for uninstall
            let data = data_storage.lock().unwrap();
            let data_cursor = Cursor::new(data.as_slice());
            let mut zip = match ZipArchive::new(data_cursor) {
                Ok(v) => v,
                Err(v) => return Err(format!("Unable to open .zip file: {:?}", v)),
            };

            let extract_base_percentage = base_package_percentage + base_package_range * 0.66;
            let extract_range_percentage = base_package_range / 3.0;

            let zip_size = zip.len();

            for i in 0..zip_size {
                let mut file = zip.by_index(i).unwrap();

                let percentage =
                    extract_base_percentage + extract_range_percentage / zip_size as f64 * i as f64;

                messages
                    .send(InstallMessage::Status(
                        format!("Extracting {} ({} of {})", file.name(), i + 1, zip_size),
                        percentage,
                    ))
                    .unwrap();

                // Create target file
                let target_path = path.join(file.name());
                println!("target_path: {:?}", target_path);

                installed_files.push(file.name().to_owned());

                // Check to make sure this isn't a directory
                if file.name().ends_with("/") || file.name().ends_with("\\") {
                    // Create this directory and move on
                    match create_dir_all(target_path) {
                        Ok(v) => v,
                        Err(v) => return Err(format!("Unable to open file: {:?}", v)),
                    }
                    continue;
                }

                match target_path.parent() {
                    Some(v) => match create_dir_all(v) {
                        Ok(v) => v,
                        Err(v) => return Err(format!("Unable to open file: {:?}", v)),
                    },
                    None => {}
                }

                let mut target_file = match File::create(target_path) {
                    Ok(v) => v,
                    Err(v) => return Err(format!("Unable to open file handle: {:?}", v)),
                };

                // Cross the streams
                match copy(&mut file, &mut target_file) {
                    Ok(v) => v,
                    Err(v) => return Err(format!("Unable to write to file: {:?}", v)),
                };
            }

            // Save metadata about this package
            // TODO: Check if this package already exists
            self.database.push(LocalInstallation {
                name: package.name.to_owned(),
                version: latest_version,
                files: installed_files,
            });

            count += 1.0;
        }

        self.save_database()?;

        // Copy installer binary to target directory
        messages
            .send(InstallMessage::Status(
                format!("Copying installer binary"),
                0.99,
            ))
            .unwrap();

        let current_app = match current_exe() {
            Ok(v) => v,
            Err(v) => return Err(format!("Unable to locate installer binary: {:?}", v)),
        };

        let mut current_app_file = match File::open(current_app) {
            Ok(v) => v,
            Err(v) => return Err(format!("Unable to open installer binary: {:?}", v)),
        };

        let platform_extension = if cfg!(windows) {
            "maintenancetool.exe"
        } else {
            "maintenancetool"
        };

        let new_app = path.join(platform_extension);

        let mut new_app_file = match File::create(new_app) {
            Ok(v) => v,
            Err(v) => return Err(format!("Unable to open installer binary: {:?}", v)),
        };

        match copy(&mut current_app_file, &mut new_app_file) {
            Err(v) => return Err(format!("Unable to copy installer binary: {:?}", v)),
            _ => {}
        };

        Ok(())
    }

    /// Saves the applications database.
    pub fn save_database(&self) -> Result<(), String> {
        // We have to have a install path for us to be able to do anything
        let path = match self.install_path.clone() {
            Some(v) => v,
            None => return Err(format!("No install directory for installer"))
        };

        let metadata_path = Path::new(&path).join("metadata.json");
        let metadata_file = match File::create(metadata_path) {
            Ok(v) => v,
            Err(v) => return Err(format!("Unable to open file handle: {:?}", v)),
        };

        match serde_json::to_writer(metadata_file, &self.database) {
            Ok(v) => v,
            Err(v) => return Err(format!("Unable to write to file: {:?}", v)),
        };

        Ok(())
    }

    /// Configures this installer to install to the specified location.
    /// If there was a currently configured install path, this will be left as-is.
    pub fn set_install_dir(&mut self, dir : &str) {
        self.install_path = Some(dir.to_owned());
    }

    /// Creates a new instance of the Installer Framework with a specified Config.
    pub fn new(config: Config) -> Self {
        InstallerFramework {
            config,
            database : Vec::new(),
            install_path : None,
            preexisting_install : false
        }
    }

    /// Creates a new instance of the Installer Framework with a specified Config, managing
    /// a pre-existing installation.
    pub fn new_with_db(config: Config, install_path : String) -> Result<Self, String> {
        let metadata_path = Path::new(&install_path).join("metadata.json");
        let metadata_file = match File::open(metadata_path) {
            Ok(v) => v,
            Err(v) => return Err(format!("Unable to open file handle: {:?}", v)),
        };

        let database : Vec<LocalInstallation> = match serde_json::from_reader(metadata_file) {
            Ok(v) => v,
            Err(v) => return Err(format!("Unable to read metadata file: {:?}", v)),
        };

        Ok(InstallerFramework {
            config,
            database,
            install_path : Some(install_path),
            preexisting_install : true
        })
    }
}
