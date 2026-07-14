use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::workspaces;
use serde::Deserialize;
use std::path::Path;

pub(super) fn resolved_dependency_name(
    specifier: &str,
    package_dir: &Path,
    workspace: &workspaces::WorkspaceMap,
) -> Option<String> {
    if let Some(path) = local_path_specifier(specifier) {
        return resolve_workspace_path(path, package_dir, workspace);
    }
    let stripped = specifier
        .strip_prefix("workspace:")
        .or_else(|| specifier.strip_prefix("npm:"))?;
    let aliased = stripped.strip_prefix("npm:").unwrap_or(stripped);
    if specifier.starts_with("workspace:") && workspace_range_specifier(aliased) {
        return None;
    }
    package_name_from_alias_specifier(aliased)
}

pub(super) fn resolved_dependency_name_with_sources(
    specifier: &str,
    package_dir: &Path,
    workspace: &workspaces::WorkspaceMap,
    _sources: &crate::codebase::ts_source::SourceStore,
) -> Option<String> {
    resolved_dependency_name(specifier, package_dir, workspace)
}

pub(super) fn workspace_dependency_name_with_sources(
    dependency_name: &str,
    specifier: &str,
    package_dir: &Path,
    workspace: &workspaces::WorkspaceMap,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Option<String> {
    if let Some(path) = local_path_specifier(specifier) {
        return resolve_workspace_path(path, package_dir, workspace);
    }
    if specifier.starts_with("workspace:") {
        return resolved_dependency_name_with_sources(specifier, package_dir, workspace, sources)
            .or_else(|| {
                workspace_has_package(dependency_name, workspace)
                    .then(|| dependency_name.to_string())
            });
    }
    workspace_has_matching_range_with_sources(dependency_name, specifier, workspace, sources)
        .then(|| dependency_name.to_string())
}

fn workspace_has_package(name: &str, workspace: &workspaces::WorkspaceMap) -> bool {
    workspace.package_by_name(name).is_some()
}

fn workspace_has_matching_range_with_sources(
    name: &str,
    specifier: &str,
    workspace: &workspaces::WorkspaceMap,
    sources: &crate::codebase::ts_source::SourceStore,
) -> bool {
    workspace
        .package_by_name(name)
        .and_then(|package| workspace_package_version_with_sources(&package.dir, sources))
        .is_some_and(|version| range_matches_version(specifier, &version))
}

#[derive(Deserialize)]
struct PackageVersion {
    version: Option<String>,
}

fn workspace_package_version_with_sources(
    package_dir: &Path,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Option<String> {
    let content = crate::codebase::rules::read_source(sources, &package_dir.join("package.json"))?;
    serde_json::from_str::<PackageVersion>(&content)
        .ok()
        .and_then(|package| package.version)
}

fn range_matches_version(range: &str, version: &str) -> bool {
    if range == version {
        return true;
    }
    let version_parts: Vec<_> = version.split('.').collect();
    let range_parts: Vec<_> = range.split('.').collect();
    !range_parts.is_empty()
        && range_parts.len() <= version_parts.len()
        && range_parts.iter().enumerate().all(|(idx, part)| {
            matches!(part.to_ascii_lowercase().as_str(), "x" | "*")
                || version_parts
                    .get(idx)
                    .is_some_and(|version| version == part)
        })
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

fn workspace_range_specifier(specifier: &str) -> bool {
    matches!(specifier.to_ascii_lowercase().as_str(), "x" | "*")
        || looks_like_semver_range(specifier)
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
        && !looks_like_semver_range(name)
        && (!name.starts_with(|c: char| c.is_ascii_digit())
            || name.chars().any(|c| c.is_ascii_alphabetic()))
}

fn looks_like_semver_range(name: &str) -> bool {
    let core = name.split_once(['-', '+']).map_or(name, |(core, _)| core);
    let mut parts = core.split('.');
    let major = parts.next().unwrap_or("");
    let Some(minor) = parts.next() else {
        return false;
    };
    !major.is_empty()
        && !minor.is_empty()
        && major.chars().all(|c| c.is_ascii_digit())
        && semver_range_part(minor)
        && parts.all(semver_range_part)
}

fn semver_range_part(part: &str) -> bool {
    !part.is_empty()
        && (part.chars().all(|c| c.is_ascii_digit())
            || matches!(part.to_ascii_lowercase().as_str(), "x" | "*"))
}

pub(super) fn workspace_path_specifier(specifier: &str) -> Option<&str> {
    let stripped = specifier.strip_prefix("workspace:")?;
    stripped.starts_with('.').then_some(stripped)
}

fn local_path_specifier(specifier: &str) -> Option<&str> {
    workspace_path_specifier(specifier).or_else(|| {
        specifier
            .strip_prefix("file:")
            .or_else(|| specifier.strip_prefix("link:"))
            .filter(|path| relative_local_path(path))
    })
}

fn relative_local_path(path: &str) -> bool {
    !path.is_empty() && !path.starts_with('/') && !path.starts_with('\\') && !path.contains("://")
}

pub(super) fn resolve_workspace_path(
    path: &str,
    base_dir: &Path,
    workspace: &workspaces::WorkspaceMap,
) -> Option<String> {
    let target_dir = normalize_path(&base_dir.join(path));
    workspace
        .package_by_dir(&target_dir)
        .map(|package| package.name.clone())
}
