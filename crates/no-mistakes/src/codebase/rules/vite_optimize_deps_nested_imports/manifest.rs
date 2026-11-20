//! Package name, declared dependencies, and published-entry imports.

use super::entry_imports::{self, RuntimeImport};
use super::specifier::package_name;
use super::RuleFinding;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(super) struct PackageFacts {
    pub name: String,
    pub declared: BTreeSet<String>,
    pub imports: Vec<RuntimeImport>,
    pub entry_file: Option<String>,
    pub unknown: Option<RuleFinding>,
}

pub(super) fn load(
    root: &Path,
    dir: &Path,
    sources: &SourceStore,
) -> Result<PackageFacts, Box<RuleFinding>> {
    let manifest = dir.join("package.json");
    let source = read_required(root, &manifest, sources)?;
    let value: Value = serde_json::from_str(&source).map_err(|error| {
        Box::new(unanalyzable(
            root,
            &manifest,
            format!("package.json is not JSON ({error})"),
        ))
    })?;
    let name = value
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .ok_or_else(|| Box::new(unanalyzable(root, &manifest, "package.json has no name")))?
        .to_string();
    let declared = dependency_names(&value)
        .map_err(|reason| Box::new(unanalyzable(root, &manifest, reason)))?;
    let Some(entry) = published_entry(&value) else {
        return Ok(PackageFacts {
            name,
            declared,
            imports: Vec::new(),
            entry_file: None,
            unknown: None,
        });
    };
    let entry_path = entry_path(dir, &entry);
    let entry_file = relative_slash_path(root, &entry_path);
    let entry_source = match sources.read_source_path(&entry_path) {
        Ok(source) => source,
        Err(error) => {
            return Ok(unknown_package(
                name,
                declared,
                super::super::source_access::read_failure(
                    root,
                    &entry_path,
                    super::RULE_ID,
                    &error,
                ),
            ));
        }
    };
    match entry_imports::collect(&entry_path, &entry_source) {
        Ok(imports) => Ok(PackageFacts {
            name,
            declared,
            imports,
            entry_file: Some(entry_file),
            unknown: None,
        }),
        Err(_) => Ok(unknown_package(
            name,
            declared,
            unanalyzable(root, &entry_path, "cannot parse published entry"),
        )),
    }
}

fn unknown_package(name: String, declared: BTreeSet<String>, finding: RuleFinding) -> PackageFacts {
    PackageFacts {
        name,
        declared,
        imports: Vec::new(),
        entry_file: None,
        unknown: Some(finding),
    }
}

fn read_required(
    root: &Path,
    path: &Path,
    sources: &SourceStore,
) -> Result<Arc<str>, Box<RuleFinding>> {
    sources.read_source_path(path).map_err(|error| {
        Box::new(super::super::source_access::read_failure(
            root,
            path,
            super::RULE_ID,
            &error,
        ))
    })
}

fn dependency_names(value: &Value) -> Result<BTreeSet<String>, String> {
    let mut names = BTreeSet::new();
    for field in ["dependencies", "peerDependencies"] {
        match value.get(field) {
            None | Some(Value::Null) => {}
            Some(Value::Object(map)) => {
                names.extend(map.keys().filter_map(|key| package_name(key)));
            }
            Some(_) => return Err(format!("{field} must be an object")),
        }
    }
    Ok(names)
}

fn published_entry(value: &Value) -> Option<String> {
    if let Some(exports) = value.get("exports") {
        if let Some(entry) = exports.as_str() {
            return Some(entry.to_string());
        }
        if let Some(entry) = exports.get(".").and_then(string_entry) {
            return Some(entry);
        }
    }
    for key in ["module", "main"] {
        if let Some(entry) = value.get(key).and_then(Value::as_str) {
            if !entry.is_empty() {
                return Some(entry.to_string());
            }
        }
    }
    None
}

fn string_entry(value: &Value) -> Option<String> {
    if let Some(entry) = value.as_str() {
        return (!entry.is_empty()).then(|| entry.to_string());
    }
    let object = value.as_object()?;
    for key in ["import", "default", "module", "require"] {
        if let Some(entry) = object.get(key).and_then(Value::as_str) {
            if !entry.is_empty() {
                return Some(entry.to_string());
            }
        }
    }
    None
}

fn entry_path(dir: &Path, entry: &str) -> PathBuf {
    let relative = entry.strip_prefix("./").unwrap_or(entry);
    crate::codebase::ts_resolver::normalize_path(&dir.join(relative))
}

fn unanalyzable(root: &Path, path: &Path, reason: impl AsRef<str>) -> RuleFinding {
    let reason = reason.as_ref();
    RuleFinding {
        causes: Vec::new(),
        rule: super::RULE_ID.to_string(),
        file: relative_slash_path(root, path),
        line: 1,
        message: format!(
            "cannot verify {}: {reason}; write a static package manifest and published entry so this check can verify nested imports",
            super::RULE_ID
        ),
        import: None,
        target: Some("unanalyzable".to_string()),
    }
}
