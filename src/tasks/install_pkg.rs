//! Installs a specific package.

use installer::InstallerFramework;

use tasks::Task;
use tasks::TaskParamType;

use config::PackageDescription;
use installer::LocalInstallation;

use std::fs::create_dir_all;
use std::fs::File;
use std::io::copy;
use std::io::Cursor;

use tasks::download_pkg::DownloadPackageTask;
use tasks::uninstall_pkg::UninstallPackageTask;

use zip::ZipArchive;

pub struct InstallPackageTask {
    pub name: String,
}

impl Task for InstallPackageTask {
    fn execute(
        &mut self,
        mut input: Vec<TaskParamType>,
        context: &mut InstallerFramework,
        messenger: &Fn(&str, f32),
    ) -> Result<TaskParamType, String> {
        messenger(&format!("Installing package {:?}...", self.name), 0.0);

        let path = context
            .install_path
            .as_ref()
            .expect("No install path specified");

        let mut installed_files = Vec::new();

        let mut metadata: Option<PackageDescription> = None;
        for description in &context.config.packages {
            if &self.name == &description.name {
                metadata = Some(description.clone());
                break;
            }
        }

        let package = match metadata {
            Some(v) => v,
            None => return Err(format!("Package {:?} could not be found.", self.name)),
        };

        let _ = input.pop().expect("Should have input from uninstaller!");

        let data = input.pop().expect("Should have input from resolver!");
        let (file, data) = match data {
            TaskParamType::FileContents(file, data) => (file, data),
            _ => return Err(format!("Unexpected param type to install package")),
        };

        // TODO: Handle files other then zips
        let data_cursor = Cursor::new(data.as_slice());
        let mut zip = match ZipArchive::new(data_cursor) {
            Ok(v) => v,
            Err(v) => return Err(format!("Unable to open .zip file: {:?}", v)),
        };

        let zip_size = zip.len();

        for i in 0..zip_size {
            let mut file = zip.by_index(i).unwrap();

            messenger(
                &format!("Extracting {} ({} of {})", file.name(), i + 1, zip_size),
                (i as f32) / (zip_size as f32),
            );

            let filename = file.name().replace("\\", "/");

            // Ensure that parent directories exist
            let mut parent_dir = &filename[..];
            while let Some(v) = parent_dir.rfind("/") {
                parent_dir = &parent_dir[0..v + 1];

                if !installed_files.contains(&parent_dir.to_string()) {
                    installed_files.push(parent_dir.to_string());
                }

                match create_dir_all(path.join(&parent_dir)) {
                    Ok(v) => v,
                    Err(v) => return Err(format!("Unable to create dir: {:?}", v)),
                }

                parent_dir = &parent_dir[0..v];
            }

            // Create target file
            let target_path = path.join(&filename);
            println!("target_path: {:?}", target_path);

            installed_files.push(filename.to_string());

            // Check to make sure this isn't a directory
            if filename.ends_with("/") || filename.ends_with("\\") {
                // Create this directory and move on
                match create_dir_all(target_path) {
                    Ok(v) => v,
                    Err(v) => return Err(format!("Unable to create dir: {:?}", v)),
                }
                continue;
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
        context.database.push(LocalInstallation {
            name: package.name.to_owned(),
            version: file,
            files: installed_files,
        });

        Ok(TaskParamType::None)
    }

    fn dependencies(&self) -> Vec<Box<Task>> {
        vec![
            Box::new(DownloadPackageTask {
                name: self.name.clone(),
            }),
            Box::new(UninstallPackageTask {
                name: self.name.clone(),
                optional: true,
            }),
        ]
    }

    fn name(&self) -> String {
        format!("InstallPackageTask (for {:?})", self.name)
    }
}
