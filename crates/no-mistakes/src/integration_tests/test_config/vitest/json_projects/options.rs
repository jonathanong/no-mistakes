use super::Options;
use crate::integration_tests::test_config::vitest::Extends;
use crate::integration_tests::types::{VitestSetupDependency, VitestSetupField};
use anyhow::{Context, Result};
use serde_json::{Map, Value};
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn parse(object: &Map<String, Value>, path: &Path) -> Result<Options> {
    Ok(Options {
        name: optional_name(object)?,
        root: optional_string(object, "root")?,
        include: optional_strings(object, "include")?,
        exclude: optional_strings(object, "exclude")?,
        setup_files: dependencies(object, "setupFiles", VitestSetupField::SetupFiles, path)?,
        global_setup: dependencies(object, "globalSetup", VitestSetupField::GlobalSetup, path)?,
        extends: optional_bool(object, "extends")?.map(|value| {
            if value {
                Extends::True
            } else {
                Extends::False
            }
        }),
        ..Options::default()
    })
}

pub(super) fn merge(base: &mut Options, nested: Options) {
    base.name = nested.name.or(base.name.take());
    base.root = nested.root.or(base.root.take());
    base.include = nested.include.or(base.include.take());
    base.exclude = nested.exclude.or(base.exclude.take());
    base.setup_files = nested.setup_files.or(base.setup_files.take());
    base.global_setup = nested.global_setup.or(base.global_setup.take());
    base.extends = nested.extends.or(base.extends.take());
}

fn optional_name(object: &Map<String, Value>) -> Result<Option<String>> {
    object
        .get("name")
        .map(|value| {
            value
                .as_str()
                .map(str::to_string)
                .or_else(|| {
                    value
                        .as_object()
                        .and_then(|name| name.get("label"))
                        .and_then(Value::as_str)
                        .map(str::to_string)
                })
                .with_context(|| {
                    "expected `name` to be a string or an object with a string `label`"
                })
        })
        .transpose()
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

fn dependencies(
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
                    needs_final_catalog_reparse: false,
                    unresolved_config_extends: None,
                    config_extends_provenance: false,
                    resolved_path: None,
                    resolution_base: base.clone(),
                    declaration_path: path.to_path_buf(),
                    declaration_line: 1,
                    trigger_paths: BTreeSet::from([path.to_path_buf()]),
                    resolver_candidate_paths: BTreeSet::new(),
                    conservative_specifiers: BTreeSet::new(),
                    transitive_trigger_paths: BTreeSet::new(),
                })
                .collect(),
        )
    })
}
