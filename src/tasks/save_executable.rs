//! Saves the installer executable into the install directory.

use installer::InstallerFramework;

use tasks::Task;
use tasks::TaskParamType;

use std::fs::File;
use std::fs::OpenOptions;

use std::io::copy;

use std::env::current_exe;

pub struct SaveExecutableTask {}

impl Task for SaveExecutableTask {
    fn execute(
        &mut self,
        input: Vec<TaskParamType>,
        context: &mut InstallerFramework,
        messenger: &Fn(&str, f32),
    ) -> Result<TaskParamType, String> {
        assert_eq!(input.len(), 0);
        messenger("Copying installer binary...", 0.0);

        let path = context
            .install_path
            .as_ref()
            .expect("No install path specified");

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

        let mut file_metadata = OpenOptions::new();
        file_metadata.write(true).create_new(true);

        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;

            file_metadata.mode(0o770);
        }

        let mut new_app_file = match file_metadata.open(new_app) {
            Ok(v) => v,
            Err(v) => return Err(format!("Unable to open installer binary: {:?}", v)),
        };

        match copy(&mut current_app_file, &mut new_app_file) {
            Err(v) => return Err(format!("Unable to copy installer binary: {:?}", v)),
            _ => {}
        };

        Ok(TaskParamType::None)
    }

    fn dependencies(&self) -> Vec<Box<Task>> {
        vec![]
    }

    fn name(&self) -> String {
        format!("SaveExecutableTask")
    }
}
