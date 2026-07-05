use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::workspaces;
use std::path::Path;

pub(super) fn resolved_dependency_name(
    specifier: &str,
    package_dir: &Path,
    workspace: &workspaces::WorkspaceMap,
) -> Option<String> {
    if let Some(path) = workspace_path_specifier(specifier) {
        return resolve_workspace_path(path, package_dir, workspace);
    }
    let stripped = specifier
        .strip_prefix("workspace:")
        .or_else(|| specifier.strip_prefix("npm:"))?;
    let aliased = stripped.strip_prefix("npm:").unwrap_or(stripped);
    package_name_from_alias_specifier(aliased)
}

pub(super) fn workspace_dependency_name(
    dependency_name: &str,
    specifier: &str,
    package_dir: &Path,
    workspace: &workspaces::WorkspaceMap,
) -> Option<String> {
    if let Some(path) = workspace_path_specifier(specifier) {
        return resolve_workspace_path(path, package_dir, workspace);
    }
    if specifier.starts_with("workspace:") {
        return resolved_dependency_name(specifier, package_dir, workspace).or_else(|| {
            workspace_has_package(dependency_name, workspace).then(|| dependency_name.to_string())
        });
    }
    (!specifier.starts_with("npm:") && workspace_has_package(dependency_name, workspace))
        .then(|| dependency_name.to_string())
}

fn workspace_has_package(name: &str, workspace: &workspaces::WorkspaceMap) -> bool {
    workspace
        .packages
        .iter()
        .any(|package| package.name == name)
}

fn package_name_from_alias_specifier(specifier: &str) -> Option<String> {
    if let Some(stripped) = specifier.strip_prefix('@') {
        let slash = stripped.find('/')?;
        let name_start = slash + 2;
        let rest = specifier.get(name_start..)?;
        let version_start = rest.find('@').unwrap_or(rest.len());
        let name = specifier.get(..name_start + version_start)?;
        return valid_package_name(name).then(|| name.to_string());
    }
    let version_start = specifier.find('@').unwrap_or(specifier.len());
    let name = specifier.get(..version_start)?;
    valid_package_name(name).then(|| name.to_string())
}

fn valid_package_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('.')
        && !name.starts_with('/')
        && !name.starts_with('*')
        && !name.starts_with('^')
        && !name.starts_with('~')
        && !name.starts_with('<')
        && !name.starts_with('>')
        && !name.starts_with('=')
        && (!name.starts_with(|c: char| c.is_ascii_digit())
            || name.chars().any(|c| c.is_ascii_alphabetic()))
}

pub(super) fn workspace_path_specifier(specifier: &str) -> Option<&str> {
    let stripped = specifier.strip_prefix("workspace:")?;
    stripped.starts_with('.').then_some(stripped)
}

pub(super) fn resolve_workspace_path(
    path: &str,
    base_dir: &Path,
    workspace: &workspaces::WorkspaceMap,
) -> Option<String> {
    let target_dir = normalize_path(&base_dir.join(path));
    workspace
        .packages
        .iter()
        .find(|package| package.dir == target_dir)
        .map(|package| package.name.clone())
}
