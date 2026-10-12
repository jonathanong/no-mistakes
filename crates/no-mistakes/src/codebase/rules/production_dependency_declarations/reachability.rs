//! Intra-package "production reachable" file closure.
//!
//! Seeds from files outside the package that import one of its files by
//! package name or subpath (excluding test-only importers), then follows
//! relative and self-reference imports that stay inside the package.

use super::specifier;
use crate::codebase::dependencies::extract::ImportKind;
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::workspaces::WorkspaceMap;
use globset::GlobSet;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

const RESOLVE_EXTENSIONS: &[&str] = &[".mts", ".ts", ".tsx", ".mjs", ".js", ".jsx", ".cts", ".cjs"];
const INDEX_BASENAMES: &[&str] = &[
    "index.mts",
    "index.ts",
    "index.tsx",
    "index.mjs",
    "index.js",
    "index.jsx",
    "index.cts",
    "index.cjs",
];

pub(super) struct FileImport {
    pub(super) line: u32,
    pub(super) specifier: String,
    pub(super) kind: ImportKind,
}

mod prepared;
pub(super) use prepared::PreparedReachability;

/// Borrow literal import facts, including recovered syntax and failure entries.
/// Missing demand is an ownership error, never permission to parse again.
pub(super) fn file_imports(
    file: &Path,
    facts: &dyn TsFactLookup,
) -> anyhow::Result<Vec<FileImport>> {
    let imports = facts.get_production_imports(file).ok_or_else(|| {
        anyhow::anyhow!(
            "missing prepared production import facts for {}",
            file.display()
        )
    })?;
    Ok(imports
        .iter()
        .filter(|import| !import.computed)
        .map(|import| FileImport {
            line: import.line,
            specifier: import.specifier.clone(),
            kind: import.kind,
        })
        .collect())
}

/// Inputs shared across every package's reachability walk within one rule
/// application, computed once for every in-scope file.
pub(super) struct ReachabilityContext<'a> {
    pub(super) root: &'a Path,
    pub(super) workspace: &'a WorkspaceMap,
    pub(super) imports_by_file: &'a HashMap<PathBuf, Vec<FileImport>>,
    pub(super) owners: &'a HashMap<PathBuf, PathBuf>,
    pub(super) test_globset: &'a GlobSet,
    pub(super) visible: &'a crate::fx::PathSet,
}

/// Files of the package rooted at `package_dir` that are reachable from
/// production entry points.
pub(super) fn production_reachable_files(
    relationships: &PreparedReachability,
    package_dir: &Path,
    package_files: &HashSet<PathBuf>,
) -> HashSet<PathBuf> {
    let mut reachable = HashSet::new();
    let mut queue = VecDeque::new();
    if let Some(seeds) = relationships.external_seeds.get(package_dir) {
        for target in seeds {
            if package_files.contains(target) && reachable.insert(target.clone()) {
                queue.push_back(target.clone());
            }
        }
    }
    while let Some(file) = queue.pop_front() {
        let Some(adjacent) = relationships.edges.forward().get(&file) else {
            continue;
        };
        for (target, _) in adjacent {
            if package_files.contains(target) && reachable.insert(target.clone()) {
                queue.push_back(target.clone());
            }
        }
    }
    reachable
}

fn resolve_target(
    workspace: &WorkspaceMap,
    file: &Path,
    import_specifier: &str,
    visible: &crate::fx::PathSet,
) -> Option<PathBuf> {
    if specifier::is_relative(import_specifier) {
        resolve_relative(file, import_specifier, visible)
    } else {
        workspace.resolve_specifier_from_file_visible(import_specifier, file, visible)
    }
}

fn resolve_relative(file: &Path, specifier: &str, visible: &crate::fx::PathSet) -> Option<PathBuf> {
    let dir = file.parent()?;
    try_resolve(&normalize_path(&dir.join(specifier)), visible)
}

fn try_resolve(candidate: &Path, visible: &crate::fx::PathSet) -> Option<PathBuf> {
    if visible.contains(candidate) {
        return Some(candidate.to_path_buf());
    }
    let base = candidate.to_string_lossy();
    for ext in RESOLVE_EXTENSIONS {
        let with_ext = PathBuf::from(format!("{base}{ext}"));
        if visible.contains(&with_ext) {
            return Some(with_ext);
        }
    }
    for name in INDEX_BASENAMES {
        let index_path = candidate.join(name);
        if visible.contains(&index_path) {
            return Some(index_path);
        }
    }
    None
}

#[cfg(test)]
mod tests;
