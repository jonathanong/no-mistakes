use crate::fx::PathSet;
use std::path::{Path, PathBuf};

pub(super) fn looks_like_repo_relative_module(specifier: &str) -> bool {
    !specifier.starts_with('.')
        && !specifier.starts_with('/')
        && !specifier.starts_with('#')
        && !specifier.starts_with('@')
        && specifier.contains('/')
}

pub(super) fn path_ends_with_module(path: &Path, module: &str) -> bool {
    let normalized = path.to_string_lossy().replace('\\', "/");
    let module = module.trim_start_matches("./").replace('\\', "/");
    normalized.ends_with(&module) || normalized.ends_with(&format!("{module}.ts"))
}

#[derive(PartialEq)]
pub(super) enum ModuleIdentity {
    Path(PathBuf),
    External(String),
}

pub(super) fn identity_from_resolver(
    resolver: &crate::codebase::ts_resolver::ImportResolver<'_>,
    specifier: &str,
    importing_file: &Path,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    visible_files: &PathSet,
    remapper: &crate::codebase::ts_source::FrozenPathRemapper,
) -> Option<ModuleIdentity> {
    let classification =
        resolver.classify_import(specifier, importing_file, workspace, visible_files);
    if let Some(path) = classification.preferred_path() {
        return remapper.remap(path).map(ModuleIdentity::Path);
    }
    (classification.is_unresolved_external() && is_external_terminal(resolver, specifier))
        .then(|| ModuleIdentity::External(specifier.to_string()))
}

pub(super) fn is_external_terminal(
    resolver: &crate::codebase::ts_resolver::ImportResolver<'_>,
    specifier: &str,
) -> bool {
    !specifier.starts_with('.')
        && !specifier.starts_with('/')
        && !specifier.starts_with('#')
        && !resolver.matches_alias(specifier)
}
