//! Uninstalls a specific package.

use installer::InstallerFramework;

use tasks::Task;
use tasks::TaskParamType;

use installer::LocalInstallation;

use std::fs::remove_dir;
use std::fs::remove_file;

pub struct UninstallPackageTask {
    pub name: String,
    pub optional: bool,
}

impl Task for UninstallPackageTask {
    fn execute(
        &mut self,
        input: Vec<TaskParamType>,
        context: &mut InstallerFramework,
        messenger: &Fn(&str, f32),
    ) -> Result<TaskParamType, String> {
        assert_eq!(input.len(), 0);

        let path = context
            .install_path
            .as_ref()
            .expect("No install path specified");

        let mut metadata: Option<LocalInstallation> = None;
        for i in 0..context.database.len() {
            if &self.name == &context.database[i].name {
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
        let mut i = 0;
        for file in package.files {
            let name = file.clone();
            let file = path.join(file);
            println!("Deleting {:?}", file);

            messenger(
                &format!("Deleting {} ({} of {})", name, i + 1, max),
                (i as f32) / (max as f32),
            );

            let result = if file.is_dir() {
                remove_dir(file)
            } else {
                remove_file(file)
            };

            match result {
                Err(v) => eprintln!("Failed to delete file: {:?}", v),
                _ => {}
            }

            i += 1;
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
