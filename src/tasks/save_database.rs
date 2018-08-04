//! Saves the main database into the installation directory.

use installer::InstallerFramework;

use tasks::Task;
use tasks::TaskParamType;

pub struct SaveDatabaseTask {}

impl Task for SaveDatabaseTask {
    fn execute(
        &mut self,
        input: Vec<TaskParamType>,
        context: &mut InstallerFramework,
        messenger: &Fn(&str, f64),
    ) -> Result<TaskParamType, String> {
        assert_eq!(input.len(), 0);
        messenger("Saving application database...", 0.0);

        context.save_database()?;

        Ok(TaskParamType::None)
    }

    fn dependencies(&self) -> Vec<Box<Task>> {
        vec![]
    }

    fn name(&self) -> String {
        "SaveDatabaseTask".to_string()
    }
}
