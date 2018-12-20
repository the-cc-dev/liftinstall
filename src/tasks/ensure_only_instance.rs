//! Verifies that this is the only running instance of the installer, and that no application is running.

use installer::InstallerFramework;

use tasks::Task;
use tasks::TaskDependency;
use tasks::TaskMessage;
use tasks::TaskParamType;

use sysinfo;
use sysinfo::get_current_pid;
use sysinfo::ProcessExt;
use sysinfo::SystemExt;

pub struct EnsureOnlyInstanceTask {}

impl Task for EnsureOnlyInstanceTask {
    fn execute(
        &mut self,
        input: Vec<TaskParamType>,
        context: &mut InstallerFramework,
        _messenger: &Fn(&TaskMessage),
    ) -> Result<TaskParamType, String> {
        assert_eq!(input.len(), 0);

        let system = sysinfo::System::new();
        let current_pid = get_current_pid();
        for (pid, process) in system.get_process_list() {
            if pid == &current_pid {
                continue;
            }

            let exe = process.exe();

            if exe.ends_with("maintenancetool.exe") || exe.ends_with("maintenancetool") {
                return Err(format!("Maintenance tool is already running!"));
            }

            for package in &context.database.packages {
                for file in &package.files {
                    if exe.ends_with(file) {
                        return Err(format!("The installed application is currently running!"));
                    }
                }
            }
        }

        Ok(TaskParamType::None)
    }

    fn dependencies(&self) -> Vec<TaskDependency> {
        vec![]
    }

    fn name(&self) -> String {
        format!("EnsureOnlyInstanceTask")
    }
}
