//! Uninstalls a set of packages.

use installer::InstallerFramework;

use tasks::Task;
use tasks::TaskParamType;

use tasks::save_database::SaveDatabaseTask;
use tasks::uninstall_pkg::UninstallPackageTask;

pub struct UninstallTask {
    pub items: Vec<String>,
}

impl Task for UninstallTask {
    fn execute(
        &mut self,
        _: Vec<TaskParamType>,
        _: &mut InstallerFramework,
        messenger: &Fn(&str, f32),
    ) -> Result<TaskParamType, String> {
        messenger("Wrapping up...", 0.0);
        Ok(TaskParamType::None)
    }

    fn dependencies(&self) -> Vec<Box<Task>> {
        let mut elements = Vec::<Box<Task>>::new();

        for item in &self.items {
            elements.push(Box::new(UninstallPackageTask {
                name: item.clone(),
                optional: false,
            }));
        }

        elements.push(Box::new(SaveDatabaseTask {}));

        elements
    }

    fn name(&self) -> String {
        format!("UninstallTask")
    }
}
