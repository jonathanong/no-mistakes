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
    has_component_suffix(&normalized, &module)
        || [".ts", ".tsx", ".js", ".jsx", ".mts", ".cts", ".mjs", ".cjs"]
            .iter()
            .any(|ext| has_component_suffix(&normalized, &format!("{module}{ext}")))
}

pub(super) fn identities_match(
    configured_specifier: &str,
    configured: Option<ModuleIdentity>,
    imported: Option<ModuleIdentity>,
) -> bool {
    match (configured, imported) {
        (Some(configured_id), Some(imported_id)) => configured_id == imported_id,
        (None, Some(ModuleIdentity::Path(imported_path)))
            if looks_like_repo_relative_module(configured_specifier) =>
        {
            path_ends_with_module(&imported_path, configured_specifier)
        }
        _ => false,
    }
}

fn has_component_suffix(path: &str, suffix: &str) -> bool {
    path == suffix || path.ends_with(&format!("/{suffix}"))
}

#[derive(Clone, PartialEq)]
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

#[cfg(test)]
#[path = "module_resolution_path/tests.rs"]
mod tests;
