//! Published-entry closure: dot entry, export targets, and relative runtime edges.

use super::entry_imports::{self, RuntimeImport};
use super::manifest::{self, entry_path, published_entry};
use super::RuleFinding;
use crate::codebase::ts_resolver::{self, TsConfig};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use serde_json::Value;
use std::collections::{BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

pub(super) fn runtime_imports(
    root: &Path,
    dir: &Path,
    manifest_value: &Value,
    own_package: &str,
    sources: &SourceStore,
    all_files: &[PathBuf],
    declared: &BTreeSet<String>,
) -> Result<Vec<RuntimeImport>, Box<RuleFinding>> {
    let mut seeds = BTreeSet::new();
    if let Some(entry) = published_entry(manifest_value) {
        seeds.insert(entry_path(dir, &entry));
    }
    if let Some(exports) = manifest_value.get("exports") {
        for file in all_files {
            let file = absolute_file(root, file);
            if !file.starts_with(dir) {
                continue;
            }
            let relative = file
                .strip_prefix(dir)
                .map(slash_relative)
                .unwrap_or_default();
            if relative.is_empty() || relative == "package.json" || relative.contains('*') {
                continue;
            }
            if ts_resolver::declarations::package_matches_runtime_export(exports, &relative) {
                seeds.insert(file);
            }
        }
    }
    let tsconfig = TsConfig::default();
    let mut pending: VecDeque<PathBuf> = seeds.into_iter().collect();
    let mut seen = BTreeSet::new();
    let mut imports = Vec::new();
    while let Some(path) = pending.pop_front() {
        if !path.starts_with(dir) || !seen.insert(path.clone()) {
            continue;
        }
        if path
            .components()
            .any(|part| part.as_os_str() == "node_modules")
        {
            continue;
        }
        let file = relative_slash_path(root, &path);
        let source = sources.read_source_path(&path).map_err(|error| {
            Box::new(super::super::source_access::read_failure(
                root,
                &path,
                super::RULE_ID,
                &error,
            ))
        })?;
        let collected = entry_imports::collect(&path, &source, own_package, &file, declared)
            .map_err(|_| {
                Box::new(manifest::unanalyzable(
                    root,
                    &path,
                    "cannot parse published entry",
                ))
            })?;
        for specifier in collected.relative {
            let Some(resolved) = ts_resolver::resolve_import(&specifier, &path, &tsconfig) else {
                continue;
            };
            let resolved = ts_resolver::normalize_path(&resolved);
            if resolved.starts_with(dir) {
                pending.push_back(resolved);
            }
        }
        imports.extend(collected.imports);
    }
    Ok(imports)
}

fn absolute_file(root: &Path, path: &Path) -> PathBuf {
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    ts_resolver::normalize_path(&joined)
}

fn slash_relative(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
