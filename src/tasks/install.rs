//! Overall hierarchy for installing a installation of the application.

use installer::InstallerFramework;

use tasks::install_dir::VerifyInstallDirTask;
use tasks::install_pkg::InstallPackageTask;
use tasks::save_database::SaveDatabaseTask;
use tasks::save_executable::SaveExecutableTask;
use tasks::uninstall_pkg::UninstallPackageTask;

use tasks::Task;
use tasks::TaskParamType;

pub struct InstallTask {
    pub items: Vec<String>,
    pub uninstall_items: Vec<String>,
    pub fresh_install: bool,
}

impl Task for InstallTask {
    fn execute(
        &mut self,
        _: Vec<TaskParamType>,
        _: &mut InstallerFramework,
        messenger: &Fn(&str, f64),
    ) -> Result<TaskParamType, String> {
        messenger("Wrapping up...", 0.0);
        Ok(TaskParamType::None)
    }

    fn dependencies(&self) -> Vec<Box<Task>> {
        let mut elements = Vec::<Box<Task>>::new();

        elements.push(Box::new(VerifyInstallDirTask {
            clean_install: self.fresh_install,
        }));

        for item in &self.items {
            elements.push(Box::new(InstallPackageTask { name: item.clone() }));
        }

        for item in &self.uninstall_items {
            elements.push(Box::new(UninstallPackageTask {
                name: item.clone(),
                optional: false,
            }));
        }

        elements.push(Box::new(SaveDatabaseTask {}));

        if self.fresh_install {
            elements.push(Box::new(SaveExecutableTask {}));
        }

        elements
    }

    fn name(&self) -> String {
        "InstallTask".to_string()
    }
}
