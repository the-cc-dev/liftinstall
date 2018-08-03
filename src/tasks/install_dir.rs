//! Verifies properties about the installation directory.

use installer::InstallerFramework;

use tasks::Task;
use tasks::TaskParamType;

use std::fs::create_dir_all;
use std::fs::read_dir;

pub struct VerifyInstallDirTask {
    pub clean_install: bool,
}

impl Task for VerifyInstallDirTask {
    fn execute(
        &mut self,
        input: Vec<TaskParamType>,
        context: &mut InstallerFramework,
        messenger: &Fn(&str, f32),
    ) -> Result<TaskParamType, String> {
        assert_eq!(input.len(), 0);
        messenger("Polling installation directory...", 0.0);

        let path = context
            .install_path
            .as_ref()
            .expect("No install path specified");

        if !path.exists() {
            create_dir_all(&path)
                .map_err(|x| format!("Failed to create install directory: {:?}", x))?;
        }

        if self.clean_install {
            let paths = read_dir(&path)
                .map_err(|x| format!("Failed to read install destination: {:?}", x))?;

            if paths.count() != 0 {
                return Err(format!("Install destination is not empty."));
            }
        }

        Ok(TaskParamType::None)
    }

    fn dependencies(&self) -> Vec<Box<Task>> {
        vec![]
    }

    fn name(&self) -> String {
        format!(
            "VerifyInstallDirTask (with clean-install = {})",
            self.clean_install
        )
    }
}
