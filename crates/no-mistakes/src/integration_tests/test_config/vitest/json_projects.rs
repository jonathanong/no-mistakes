use super::{project_arrays, to_project, Options};
use crate::codebase::ts_resolver::ImportResolution;
use crate::integration_tests::types::{ConfigProject, VitestSetupDependency, VitestSetupField};
use anyhow::{Context, Result};
use serde_json::{Map, Value};
use std::collections::BTreeSet;
use std::path::Path;

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
    let mut included = BTreeSet::new();
    let mut excluded = BTreeSet::new();
    for (index, entry) in entries.iter().enumerate() {
        if let Some(specifier) = entry.as_str() {
            let (specifier, paths) = match specifier.strip_prefix('!') {
                Some(specifier) => (specifier, &mut excluded),
                None => (specifier, &mut included),
            };
            paths.extend(project_arrays::string_project_paths_with_resolver(
                specifier, path, resolver,
            ));
            continue;
        }
        let object = entry
            .as_object()
            .with_context(|| format!("expected project object at index {index}"))?;
        let mut options = parse_options(object, path)?;
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
            merge_json_options(&mut options, parse_options(test, path)?);
        }
        projects.push(to_project(config_dir, root, options, resolver));
    }
    included.retain(|path| !excluded.contains(path));
    let mut seen = BTreeSet::new();
    for project_path in included {
        if let Some(options) =
            project_arrays::parse_string_project_with_resolver(&project_path, resolver, &mut seen)?
        {
            projects.push(to_project(config_dir, root, options, resolver));
        }
    }
    Ok(projects)
}

fn parse_options(object: &Map<String, Value>, path: &Path) -> Result<Options> {
    Ok(Options {
        name: optional_string(object, "name")?,
        root: optional_string(object, "root")?,
        include: optional_strings(object, "include")?,
        exclude: optional_strings(object, "exclude")?,
        setup_files: setup_dependencies(object, "setupFiles", VitestSetupField::SetupFiles, path)?,
        global_setup: setup_dependencies(
            object,
            "globalSetup",
            VitestSetupField::GlobalSetup,
            path,
        )?,
        extends: optional_bool(object, "extends")?,
        ..Options::default()
    })
}

fn merge_json_options(base: &mut Options, nested: Options) {
    base.name = nested.name.or(base.name.take());
    base.root = nested.root.or(base.root.take());
    base.include = nested.include.or(base.include.take());
    base.exclude = nested.exclude.or(base.exclude.take());
    base.setup_files = nested.setup_files.or(base.setup_files.take());
    base.global_setup = nested.global_setup.or(base.global_setup.take());
    base.extends = nested.extends.or(base.extends);
}

fn optional_string(object: &Map<String, Value>, key: &str) -> Result<Option<String>> {
    object
        .get(key)
        .map(|value| {
            value
                .as_str()
                .map(str::to_string)
                .with_context(|| format!("expected `{key}` to be a string"))
        })
        .transpose()
}

fn optional_bool(object: &Map<String, Value>, key: &str) -> Result<Option<bool>> {
    object
        .get(key)
        .map(|value| {
            value
                .as_bool()
                .with_context(|| format!("expected `{key}` to be a boolean"))
        })
        .transpose()
}

fn optional_strings(object: &Map<String, Value>, key: &str) -> Result<Option<Vec<String>>> {
    object.get(key).map(|value| strings(value, key)).transpose()
}

fn strings(value: &Value, key: &str) -> Result<Vec<String>> {
    if let Some(value) = value.as_str() {
        return Ok(vec![value.to_string()]);
    }
    value
        .as_array()
        .with_context(|| format!("expected `{key}` to be a string or string array"))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_string)
                .with_context(|| format!("expected `{key}` entries to be strings"))
        })
        .collect()
}

fn setup_dependencies(
    object: &Map<String, Value>,
    key: &str,
    field: VitestSetupField,
    path: &Path,
) -> Result<Option<Vec<VitestSetupDependency>>> {
    let Some(value) = object.get(key) else {
        return Ok(None);
    };
    let base = path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    strings(value, key).map(|values| {
        Some(
            values
                .into_iter()
                .map(|specifier| VitestSetupDependency {
                    field,
                    specifier: Some(specifier),
                    resolved_path: None,
                    resolution_base: base.clone(),
                    declaration_path: path.to_path_buf(),
                    declaration_line: 1,
                    trigger_paths: BTreeSet::from([path.to_path_buf()]),
                    resolver_candidate_paths: BTreeSet::new(),
                    transitive_trigger_paths: BTreeSet::new(),
                })
                .collect(),
        )
    })
}
