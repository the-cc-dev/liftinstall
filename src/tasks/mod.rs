use std::any::Any;

use std::fmt;
use std::fmt::Display;

/// Allows for any time to be used as an Input to a Task.
type AnyInput = Box<Any>;

/// A Task is a small, async task conforming to a fixed set of inputs/outputs.
pub trait Task {
    type Input;
    type Output;
    type Error;

    /// Executes this individual task, evaluating to the given Output result.
    ///
    /// Each dependency is given an indice in the inputted vector.
    fn execute(
        &mut self,
        input: Vec<Self::Input>,
        messenger: &Fn(&str, f32),
    ) -> Result<Self::Output, Self::Error>;

    /// Returns a vector containing all dependencies that need to be executed
    /// before this task can function.
    fn dependencies(
        &self,
    ) -> Vec<Box<Task<Input = AnyInput, Output = Self::Input, Error = Self::Error>>>;

    /// Returns a short name used for formatting the dependency tree.
    fn name(&self) -> String;
}

/// The dependency tree allows for smart iteration on a Task struct.
pub struct DependencyTree<I, O, E> {
    task: Box<Task<Input = I, Output = O, Error = E>>,
    dependencies: Vec<DependencyTree<AnyInput, I, E>>,
}

impl<I, O, E> DependencyTree<I, O, E> {
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
    pub fn execute(&mut self, messenger: &Fn(&str, f32)) -> Result<O, E> {
        let total_tasks = (self.dependencies.len() + 1) as f32;

        let mut inputs = Vec::<I>::with_capacity(self.dependencies.len());

        let mut count = 0;

        for i in &mut self.dependencies {
            inputs.push(i.execute(&|msg: &str, progress: f32| {
                messenger(
                    msg,
                    progress / total_tasks + (1.0 / total_tasks) * count as f32,
                )
            })?);
            count += 1;
        }

        self.task.execute(inputs, &|msg: &str, progress: f32| {
            messenger(
                msg,
                progress / total_tasks + (1.0 / total_tasks) * count as f32,
            )
        })
    }

    /// Builds a new pipeline from the specified task, iterating on dependencies.
    pub fn build(task: Box<Task<Input = I, Output = O, Error = E>>) -> DependencyTree<I, O, E> {
        let dependencies = task
            .dependencies()
            .into_iter()
            .map(|x| DependencyTree::build(x))
            .collect();

        DependencyTree { task, dependencies }
    }
}

impl<I, O, E> Display for DependencyTree<I, O, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.render())
    }
}
