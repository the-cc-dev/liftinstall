//! installer.rs
//!
//! Contains the main installer structure, as well as high-level means of controlling it.

use serde_json;

use std::fs::File;

use std::env::var;

use std::path::Path;
use std::path::PathBuf;

use std::sync::mpsc::Sender;

use config::Config;

use sources::types::Version;

use tasks::install::InstallTask;
use tasks::uninstall::UninstallTask;
use tasks::DependencyTree;

use logging::LoggingErrors;

use dirs::home_dir;

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
    pub config: Config,
    pub database: Vec<LocalInstallation>,
    pub install_path: Option<PathBuf>,
    pub preexisting_install: bool,
}

/// Contains basic properties on the status of the session. Subset of InstallationFramework.
#[derive(Serialize)]
pub struct InstallationStatus {
    pub database: Vec<LocalInstallation>,
    pub install_path: Option<String>,
    pub preexisting_install: bool,
}

/// Tracks the state of a local installation
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocalInstallation {
    pub name: String,
    pub version: Version,
    pub files: Vec<String>,
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
    /// items: Array of named packages to be installed/kept
    /// messages: Channel used to send progress messages
    /// fresh_install: If the install directory must be empty
    pub fn install(
        &mut self,
        items: Vec<String>,
        messages: &Sender<InstallMessage>,
        fresh_install: bool,
    ) -> Result<(), String> {
        info!(
            "Framework: Installing {:?} to {:?}",
            items,
            self.install_path
                .clone()
                .log_expect("Install directory not initialised")
        );

        // Calculate packages to *uninstall*
        let mut uninstall_items = Vec::new();
        if !fresh_install {
            for package in &self.database {
                if !items.contains(&package.name) {
                    uninstall_items.push(package.name.clone());
                }
            }

            info!(
                "Framework: Uninstalling {:?} additionally.",
                uninstall_items
            );
        }

        let task = Box::new(InstallTask {
            items,
            uninstall_items,
            fresh_install,
        });

        let mut tree = DependencyTree::build(task);

        info!("Dependency tree:\n{}", tree);

        tree.execute(self, &|msg: &str, progress: f64| {
            if let Err(v) = messages.send(InstallMessage::Status(msg.to_string(), progress as _)) {
                error!("Failed to submit queue message: {:?}", v);
            }
        }).map(|_x| ())
    }

    /// Sends a request for everything to be uninstalled.
    pub fn uninstall(&mut self, messages: &Sender<InstallMessage>) -> Result<(), String> {
        // TODO: Cleanup maintenance tool

        let items: Vec<String> = self.database.iter().map(|x| x.name.clone()).collect();

        let task = Box::new(UninstallTask { items });

        let mut tree = DependencyTree::build(task);

        info!("Dependency tree:\n{}", tree);

        tree.execute(self, &|msg: &str, progress: f64| {
            if let Err(v) = messages.send(InstallMessage::Status(msg.to_string(), progress as _)) {
                error!("Failed to submit queue message: {:?}", v);
            }
        }).map(|_x| ())
    }

    /// Saves the applications database.
    pub fn save_database(&self) -> Result<(), String> {
        // We have to have a install path for us to be able to do anything
        let path = match self.install_path.clone() {
            Some(v) => v,
            None => return Err("No install directory for installer".to_string()),
        };

        let metadata_path = path.join("metadata.json");
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
    pub fn set_install_dir(&mut self, dir: &str) {
        self.install_path = Some(Path::new(dir).to_owned());
    }

    /// Returns metadata on the current status of the installation.
    pub fn get_installation_status(&self) -> InstallationStatus {
        InstallationStatus {
            database: self.database.clone(),
            install_path: match self.install_path.clone() {
                Some(v) => Some(v.display().to_string()),
                None => None,
            },
            preexisting_install: self.preexisting_install,
        }
    }

    /// Creates a new instance of the Installer Framework with a specified Config.
    pub fn new(config: Config) -> Self {
        InstallerFramework {
            config,
            database: Vec::new(),
            install_path: None,
            preexisting_install: false,
        }
    }

    /// Creates a new instance of the Installer Framework with a specified Config, managing
    /// a pre-existing installation.
    pub fn new_with_db(config: Config, install_path: &Path) -> Result<Self, String> {
        let path = install_path.to_owned();
        let metadata_path = path.join("metadata.json");
        let metadata_file = match File::open(metadata_path) {
            Ok(v) => v,
            Err(v) => return Err(format!("Unable to open file handle: {:?}", v)),
        };

        let database: Vec<LocalInstallation> = match serde_json::from_reader(metadata_file) {
            Ok(v) => v,
            Err(v) => return Err(format!("Unable to read metadata file: {:?}", v)),
        };

        Ok(InstallerFramework {
            config,
            database,
            install_path: Some(path),
            preexisting_install: true,
        })
    }
}
