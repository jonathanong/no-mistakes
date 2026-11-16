//! Resolve one registry specifier to the module the binding names.
use super::home::Locate;
use crate::codebase::dependencies::graph::{DepGraph, EdgeKind, NodeId};
use crate::codebase::ts_resolver::{normalize_path, ImportResolver, TsConfig};
use crate::codebase::workspaces::IndexedWorkspaceMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(super) fn resolve(locate: &Locate<'_>, from: &Path, specifier: &str) -> Option<PathBuf> {
    let imports = direct_imports(locate.graph, from);
    if let Some(path) = classified(locate, from, specifier) {
        if locate.facts.get_ts_facts(&path).is_some() {
            return Some(path);
        }
        if let Some(file) = imports.iter().find(|file| normalize_path(file) == path) {
            return Some(file.clone());
        }
    }
    imports
        .into_iter()
        .find(|file| specifier_matches(from, specifier, file))
}

fn classified(locate: &Locate<'_>, from: &Path, specifier: &str) -> Option<PathBuf> {
    let fallback_config = TsConfig::default();
    let config = locate
        .resolution
        .catalog
        .map(|catalog| catalog.config_for(from))
        .unwrap_or(&fallback_config);
    let fallback_workspace = IndexedWorkspaceMap::default();
    let workspace = locate.resolution.workspace.unwrap_or(&fallback_workspace);
    ImportResolver::new(config)
        .classify_import(specifier, from, workspace, locate.visible)
        .preferred_path()
        .map(normalize_path)
}

fn specifier_matches(from: &Path, specifier: &str, target: &Path) -> bool {
    from.parent()
        .filter(|_| specifier.starts_with("./") || specifier.starts_with("../"))
        .is_some_and(|dir| normalize_path(&dir.join(specifier)) == normalize_path(target))
}

fn direct_imports(graph: &DepGraph, file: &Path) -> Vec<PathBuf> {
    let kinds = [EdgeKind::Import, EdgeKind::WorkspaceImport]
        .into_iter()
        .collect::<HashSet<_>>();
    graph
        .deps_of(&[NodeId::file(file)], Some(1), Some(&kinds))
        .into_iter()
        .filter_map(|entry| entry.node.as_file().map(Path::to_path_buf))
        .collect()
}
