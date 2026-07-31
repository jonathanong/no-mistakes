//! Intra-package "production reachable" file closure.
//!
//! Seeds from files outside the package that import one of its files by
//! package name or subpath (excluding test-only importers), then follows
//! relative and self-reference imports that stay inside the package.

use super::specifier;
use crate::codebase::dependencies::extract::{is_tsx_file, ImportExtractor, ImportKind};
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::relative_slash_path;
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
];

pub(super) struct FileImport {
    pub(super) line: u32,
    pub(super) specifier: String,
    pub(super) kind: ImportKind,
}

/// Extract every import specifier from `file`, tagged with its syntax kind
/// and line number. Returns an empty list for unreadable or unparsable
/// sources, matching every other consumer of `ImportExtractor`.
pub(super) fn file_imports(
    file: &Path,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<FileImport> {
    let Some(source) = crate::codebase::rules::read_source(sources, file) else {
        return Vec::new();
    };
    let extractor = if is_tsx_file(file) {
        ImportExtractor::for_tsx()
    } else {
        ImportExtractor::for_typescript()
    };
    extractor
        .and_then(|extractor| extractor.extract(&source))
        .unwrap_or_default()
        .into_iter()
        .map(|import| FileImport {
            line: import.line,
            specifier: import.specifier,
            kind: import.kind,
        })
        .collect()
}

/// Inputs shared across every package's reachability walk within one rule
/// application, computed once for every in-scope file.
pub(super) struct ReachabilityContext<'a> {
    pub(super) root: &'a Path,
    pub(super) workspace: &'a WorkspaceMap,
    pub(super) imports_by_file: &'a HashMap<PathBuf, Vec<FileImport>>,
    pub(super) owners: &'a HashMap<PathBuf, PathBuf>,
    pub(super) test_globset: &'a GlobSet,
    pub(super) visible: &'a HashSet<PathBuf>,
}

/// Files of the package rooted at `package_dir` that are reachable from
/// production entry points.
pub(super) fn production_reachable_files(
    ctx: &ReachabilityContext,
    package_dir: &Path,
    package_files: &HashSet<PathBuf>,
) -> HashSet<PathBuf> {
    let mut reachable = HashSet::new();
    let mut queue = VecDeque::new();

    for (file, imports) in ctx.imports_by_file {
        if ctx.owners.get(file).map(PathBuf::as_path) == Some(package_dir) {
            continue; // an importer inside the package is not an external seed
        }
        if ctx
            .test_globset
            .is_match(relative_slash_path(ctx.root, file))
        {
            continue; // test-only importers never make a file production-reachable
        }
        for target in resolved_targets(ctx.workspace, file, imports, ctx.visible) {
            if package_files.contains(&target) && reachable.insert(target.clone()) {
                queue.push_back(target);
            }
        }
    }

    while let Some(file) = queue.pop_front() {
        let Some(imports) = ctx.imports_by_file.get(&file) else {
            continue;
        };
        for target in resolved_targets(ctx.workspace, &file, imports, ctx.visible) {
            if package_files.contains(&target) && reachable.insert(target.clone()) {
                queue.push_back(target);
            }
        }
    }

    reachable
}

fn resolved_targets(
    workspace: &WorkspaceMap,
    file: &Path,
    imports: &[FileImport],
    visible: &HashSet<PathBuf>,
) -> Vec<PathBuf> {
    imports
        .iter()
        .filter(|import| import.kind != ImportKind::Type)
        .filter_map(|import| resolve_target(workspace, file, &import.specifier, visible))
        .collect()
}

fn resolve_target(
    workspace: &WorkspaceMap,
    file: &Path,
    import_specifier: &str,
    visible: &HashSet<PathBuf>,
) -> Option<PathBuf> {
    if specifier::is_relative(import_specifier) {
        resolve_relative(file, import_specifier, visible)
    } else {
        workspace.resolve_specifier_from_file_visible(import_specifier, file, visible)
    }
}

fn resolve_relative(file: &Path, specifier: &str, visible: &HashSet<PathBuf>) -> Option<PathBuf> {
    let dir = file.parent()?;
    try_resolve(&normalize_path(&dir.join(specifier)), visible)
}

fn try_resolve(candidate: &Path, visible: &HashSet<PathBuf>) -> Option<PathBuf> {
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
