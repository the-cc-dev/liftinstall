//! Contains a framework for the processing of discrete Tasks, as well as
//! various implementations of it for installer-related tasks.

use std::fmt;
use std::fmt::Display;

use installer::InstallerFramework;

use sources::types::File;
use sources::types::Version;

pub mod download_pkg;
pub mod install;
pub mod install_dir;
pub mod install_pkg;
pub mod resolver;
pub mod save_database;
pub mod save_executable;
pub mod uninstall;
pub mod uninstall_pkg;

/// An abstraction over the various paramaters that can be passed around.
pub enum TaskParamType {
    None,
    File(Version, File),
    FileContents(Version, Vec<u8>),
}

/// A Task is a small, async task conforming to a fixed set of inputs/outputs.
pub trait Task {
    /// Executes this individual task, evaluating to the given Output result.
    ///
    /// Each dependency is given an indice in the inputted vector.
    fn execute(
        &mut self,
        input: Vec<TaskParamType>,
        context: &mut InstallerFramework,
        messenger: &Fn(&str, f32),
    ) -> Result<TaskParamType, String>;

    /// Returns a vector containing all dependencies that need to be executed
    /// before this task can function.
    fn dependencies(&self) -> Vec<Box<Task>>;

    /// Returns a short name used for formatting the dependency tree.
    fn name(&self) -> String;
}

/// The dependency tree allows for smart iteration on a Task struct.
pub struct DependencyTree {
    task: Box<Task>,
    dependencies: Vec<DependencyTree>,
}

impl DependencyTree {
    /// Renders the dependency tree into a user-presentable string.
    fn render(&self) -> String {
        let mut buf = self.task.name();

        buf += "\n";

        for i in 0..self.dependencies.len() {
            let dependencies = self.dependencies[i].render();
            let dependencies = dependencies.trim();

            if i + 1 == self.dependencies.len() {
                buf += "└── ";
                buf += &dependencies.replace("\n", "\n    ");
            } else {
                buf += "├── ";
                buf += &dependencies.replace("\n", "\n│   ");
                buf += "\n";
            }
        }

        buf
    }

    /// Executes this pipeline.
    pub fn execute(
        &mut self,
        context: &mut InstallerFramework,
        messenger: &Fn(&str, f32),
    ) -> Result<TaskParamType, String> {
        let total_tasks = (self.dependencies.len() + 1) as f32;

        let mut inputs = Vec::<TaskParamType>::with_capacity(self.dependencies.len());

        let mut count = 0;

        for i in &mut self.dependencies {
            inputs.push(i.execute(context, &|msg: &str, progress: f32| {
                messenger(
                    msg,
                    progress / total_tasks + (1.0 / total_tasks) * count as f32,
                )
            })?);
            count += 1;
        }

        self.task
            .execute(inputs, context, &|msg: &str, progress: f32| {
                messenger(
                    msg,
                    progress / total_tasks + (1.0 / total_tasks) * count as f32,
                )
            })
    }

    /// Builds a new pipeline from the specified task, iterating on dependencies.
    pub fn build(task: Box<Task>) -> DependencyTree {
        let dependencies = task
            .dependencies()
            .into_iter()
            .map(|x| DependencyTree::build(x))
            .collect();

        DependencyTree { task, dependencies }
    }
}

impl Display for DependencyTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.render())
    }
}
