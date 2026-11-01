#[path = "script_entry_commands.rs"]
mod commands;

use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::SourceStore;
use crate::fx::{fx_set, FxHashSet};
use std::path::{Path, PathBuf};

/// Source files a package script passes to a bundler.
pub(super) struct ScriptEntries {
    paths: FxHashSet<PathBuf>,
}

impl ScriptEntries {
    pub(super) fn collect(root: &Path, files: &[PathBuf]) -> Self {
        let root = normalize_path(root);
        let mut paths = fx_set();
        for manifest in manifests(&root, files) {
            if let Some(directory) = manifest.parent() {
                record_manifest(&mut paths, directory, &root, &manifest);
            }
        }
        Self { paths }
    }

    pub(super) fn contains(&self, file: &Path) -> bool {
        self.paths.contains(&normalize_path(file))
    }

    pub(super) fn exempts(&self, file: &Path, framework: bool) -> bool {
        self.contains(file) || framework
    }
}

fn record_manifest(paths: &mut FxHashSet<PathBuf>, directory: &Path, root: &Path, manifest: &Path) {
    for script in read_scripts(manifest) {
        for segment in commands::command_segments(&script) {
            for raw in commands::positional_entries(&segment) {
                if let Some(path) = resolve_entry(directory, root, &raw) {
                    paths.insert(path);
                }
            }
        }
    }
}

fn manifests(root: &Path, files: &[PathBuf]) -> Vec<PathBuf> {
    let mut directories = FxHashSet::default();
    directories.insert(root.to_path_buf());
    for file in files {
        let mut directory = normalize_path(file);
        while directory.pop() && directory.starts_with(root) {
            let done = directory == root;
            directories.insert(directory.clone());
            if done {
                break;
            }
        }
    }
    let mut found: Vec<_> = directories
        .into_iter()
        .map(|directory| normalize_path(&directory.join("package.json")))
        .filter(|manifest| manifest.is_file())
        .collect();
    found.sort();
    found
}

fn read_scripts(manifest: &Path) -> Vec<String> {
    let Some(value) = SourceStore::parse_json_optional(None, manifest) else {
        return Vec::new();
    };
    let Some(scripts) = value.get("scripts").and_then(|value| value.as_object()) else {
        return Vec::new();
    };
    scripts
        .values()
        .filter_map(|value| value.as_str().map(str::to_string))
        .collect()
}

fn is_static(word: &str) -> bool {
    !word
        .chars()
        .any(|character| matches!(character, '$' | '`' | '*' | '?' | '[' | ']' | '{' | '}'))
}

fn resolve_entry(package_dir: &Path, root: &Path, raw: &str) -> Option<PathBuf> {
    if !is_static(raw) {
        return None;
    }
    let raw_path = Path::new(raw);
    let joined = if raw_path.is_absolute() {
        raw_path.to_path_buf()
    } else {
        package_dir.join(raw_path)
    };
    let path = normalize_path(&joined);
    (path.starts_with(root) && path.is_file()).then_some(path)
}

#[cfg(test)]
#[path = "script_entries_tests.rs"]
mod tests;
