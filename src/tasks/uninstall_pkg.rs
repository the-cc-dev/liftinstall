//! Uninstalls a specific package.

use installer::InstallerFramework;

use tasks::Task;
use tasks::TaskParamType;

use installer::LocalInstallation;

use std::fs::remove_dir;
use std::fs::remove_file;

use logging::LoggingErrors;

pub struct UninstallPackageTask {
    pub name: String,
    pub optional: bool,
}

impl Task for UninstallPackageTask {
    fn execute(
        &mut self,
        input: Vec<TaskParamType>,
        context: &mut InstallerFramework,
        messenger: &Fn(&str, f64),
    ) -> Result<TaskParamType, String> {
        assert_eq!(input.len(), 0);

        let path = context
            .install_path
            .as_ref()
            .log_expect("No install path specified");

        let mut metadata: Option<LocalInstallation> = None;
        for i in 0..context.database.len() {
            if self.name == context.database[i].name {
                metadata = Some(context.database.remove(i));
                break;
            }
        }

        let mut package = match metadata {
            Some(v) => v,
            None => {
                if self.optional {
                    return Ok(TaskParamType::None);
                }

                return Err(format!(
                    "Package {:?} could not be found for uninstall.",
                    self.name
                ));
            }
        };

        messenger(&format!("Uninstalling package {:?}...", self.name), 0.0);

        // Reverse, as to delete directories last
        package.files.reverse();

        let max = package.files.len();
        for (i, file) in package.files.iter().enumerate() {
            let name = file.clone();
            let file = path.join(file);
            info!("Deleting {:?}", file);

            messenger(
                &format!("Deleting {} ({} of {})", name, i + 1, max),
                (i as f64) / (max as f64),
            );

            let result = if file.is_dir() {
                remove_dir(file)
            } else {
                remove_file(file)
            };

            if let Err(v) = result {
                error!("Failed to delete file: {:?}", v);
            }
        }

        Ok(TaskParamType::None)
    }

    fn dependencies(&self) -> Vec<Box<Task>> {
        vec![]
    }

    fn name(&self) -> String {
        format!(
            "UninstallPackageTask (for {:?}, optional = {})",
            self.name, self.optional
        )
    }
}
