//! Uninstalls a specific package.

use installer::InstallerFramework;

use tasks::Task;
use tasks::TaskParamType;

pub struct UninstallPackageTask {
    pub name: String,
    pub optional: bool,
}

impl Task for UninstallPackageTask {
    fn execute(
        &mut self,
        _: Vec<TaskParamType>,
        _: &mut InstallerFramework,
        messenger: &Fn(&str, f32),
    ) -> Result<TaskParamType, String> {
        messenger(&format!("Uninstalling package {:?}...", self.name), 0.0);

        // TODO: Find files to uninstall, wipe them out, then clean up DB

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
