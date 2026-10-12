use crate::codebase::package_deps;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) packages: Vec<String>,
    pub(crate) forbidden: Vec<String>,
    pub(crate) dependency_types: Vec<String>,
    pub(crate) lockfile: Option<PathBuf>,
}

pub(super) fn dependency_types(opts: &Options) -> std::result::Result<Vec<&str>, String> {
    if opts.dependency_types.is_empty() {
        return Ok(package_deps::PRODUCTION_DEPENDENCY_FIELDS.to_vec());
    }
    let mut validated = Vec::new();
    for field in &opts.dependency_types {
        if !package_deps::ALL_DEPENDENCY_FIELDS.contains(&field.as_str()) {
            return Err(format!(
                "dependencyTypes supports dependencies, devDependencies, peerDependencies, and optionalDependencies only; unsupported dependency type '{field}'"
            ));
        }
        validated.push(field.as_str());
    }
    Ok(validated)
}
