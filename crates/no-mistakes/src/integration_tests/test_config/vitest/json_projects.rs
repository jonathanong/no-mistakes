use super::{project_arrays, to_project, Options};
use crate::codebase::ts_resolver::ImportResolution;
use crate::integration_tests::types::ConfigProject;
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::Path;

mod options;

pub(super) fn parse(
    source: &str,
    path: &Path,
    config_dir: &Path,
    root: &Path,
    resolver: &dyn ImportResolution,
) -> Result<Vec<ConfigProject>> {
    let value: Value = serde_json::from_str(source)
        .with_context(|| format!("parsing Vitest project array {}", path.display()))?;
    let entries = value.as_array().context("expected a JSON project array")?;
    let mut projects = Vec::new();
    let mut included_paths = BTreeSet::new();
    let mut excluded_paths = BTreeSet::new();
    let mut included_roots = BTreeSet::new();
    let mut excluded_roots = BTreeSet::new();
    for (index, entry) in entries.iter().enumerate() {
        if let Some(specifier) = entry.as_str() {
            let (specifier, paths, roots) = match specifier.strip_prefix('!') {
                Some(specifier) => (specifier, &mut excluded_paths, &mut excluded_roots),
                None => (specifier, &mut included_paths, &mut included_roots),
            };
            paths.extend(project_arrays::string_project_paths_with_resolver(
                specifier, path, resolver,
            ));
            roots.extend(project_arrays::string_project_roots_with_resolver(
                specifier, path, resolver,
            ));
            continue;
        }
        let object = entry
            .as_object()
            .with_context(|| format!("expected project object at index {index}"))?;
        let mut options = options::parse(object, path)?;
        if let Some(test) = object.get("test") {
            let test = test
                .as_object()
                .with_context(|| format!("expected `test` object at index {index}"))?;
            // Match Vitest's TS/JS object semantics: once `test` exists, these
            // fields belong to that nested object. `root` and `extends` may
            // still be declared on the outer project object.
            options.name = None;
            options.include = None;
            options.exclude = None;
            options.setup_files = None;
            options.global_setup = None;
            options::merge(&mut options, options::parse(test, path)?);
        }
        projects.push(to_project(config_dir, root, options, resolver));
    }
    included_paths.retain(|path| !excluded_paths.contains(path));
    included_roots.retain(|root| !excluded_roots.contains(root));
    let mut seen = BTreeSet::new();
    for project_path in included_paths {
        if let Some(options) =
            project_arrays::parse_string_project_with_resolver(&project_path, resolver, &mut seen)?
        {
            projects.push(to_project(config_dir, root, options, resolver));
        }
    }
    for project_root in included_roots {
        projects.push(to_project(
            config_dir,
            root,
            Options {
                root: Some(project_root.to_string_lossy().into_owned()),
                // JSON workspace folder strings are independent projects too.
                standalone_config: true,
                ..Options::default()
            },
            resolver,
        ));
    }
    Ok(projects)
}
