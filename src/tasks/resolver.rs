//! Resolves package names into a metadata + version object.

use std::env::consts::OS;

use installer::InstallerFramework;

use tasks::Task;
use tasks::TaskParamType;

use config::PackageDescription;

use regex::Regex;

pub struct ResolvePackageTask {
    pub name: String,
}

impl Task for ResolvePackageTask {
    fn execute(
        &mut self,
        input: Vec<TaskParamType>,
        context: &mut InstallerFramework,
        messenger: &Fn(&str, f32),
    ) -> Result<TaskParamType, String> {
        assert_eq!(input.len(), 0);
        let mut metadata: Option<PackageDescription> = None;
        for description in &context.config.packages {
            if &self.name == &description.name {
                metadata = Some(description.clone());
                break;
            }
        }

        let package = match metadata {
            Some(v) => v,
            None => return Err(format!("Package {:?} could not be found.", self.name)),
        };

        messenger(
            &format!(
                "Polling {} for latest version of {:?}...",
                package.source.name, package.name
            ),
            0.0,
        );

        let results = package.source.get_current_releases()?;

        messenger(
            &format!("Resolving dependency for {:?}...", package.name),
            0.5,
        );

        let filtered_regex = package.source.match_regex.replace("#PLATFORM#", OS);
        let regex = match Regex::new(&filtered_regex) {
            Ok(v) => v,
            Err(v) => return Err(format!("An error occured while compiling regex: {:?}", v)),
        };

        // Find the latest release in here
        let latest_result = results
            .into_iter()
            .filter(|f| f.files.iter().filter(|x| regex.is_match(&x.name)).count() > 0)
            .max_by_key(|f| f.version.clone());

        let latest_result = match latest_result {
            Some(v) => v,
            None => return Err(format!("No release with correct file found")),
        };

        let latest_version = latest_result.version.clone();

        // Find the matching file in here
        let latest_file = latest_result
            .files
            .into_iter()
            .filter(|x| regex.is_match(&x.name))
            .next()
            .unwrap();

        println!("Selected file: {:?}", latest_file);

        Ok(TaskParamType::File(latest_version, latest_file))
    }

    fn dependencies(&self) -> Vec<Box<Task>> {
        vec![]
    }

    fn name(&self) -> String {
        format!("ResolvePackageTask (for {:?})", self.name)
    }
}