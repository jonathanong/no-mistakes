use super::range::version_satisfies;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::workspaces::{WorkspaceMap, WorkspacePackage};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub(super) fn workspace_dependency_names(
    package_dir: &Path,
    dependency_types: &[&str],
    workspace: &WorkspaceMap,
    sources: &crate::codebase::ts_source::SourceStore,
) -> BTreeSet<String> {
    crate::codebase::package_deps::dependency_entries_from_source_store(
        &package_dir.join("package.json"),
        dependency_types,
        sources,
    )
    .into_iter()
    .filter(|entry| {
        is_workspace_edge(
            &entry.name,
            &entry.specifier,
            package_dir,
            workspace,
            sources,
        )
    })
    .map(|entry| entry.name)
    .collect()
}

/// A dependency is a workspace edge unless the specifier is proven to resolve
/// outside the workspace. `workspace:` always names the workspace package.
/// `file:`/`link:`/`portal:` count when the path is that package's directory.
/// A version range counts when the package version satisfies it and the
/// specifier is not an explicit non-workspace resolution (`catalog:`, `npm:`,
/// git, or a URL). That is the npm/Yarn link and pnpm `linkWorkspacePackages`
/// match: an unsatisfied range is the registry resolution and is skipped.
fn is_workspace_edge(
    name: &str,
    specifier: &str,
    package_dir: &Path,
    workspace: &WorkspaceMap,
    sources: &crate::codebase::ts_source::SourceStore,
) -> bool {
    let Some(named) = workspace.package_by_name(name) else {
        return false;
    };
    if let Some(rest) = specifier.strip_prefix("workspace:") {
        return workspace_protocol_edge(rest, named, package_dir);
    }
    if let Some(path) = local_protocol_path(specifier) {
        return path_is_package_dir(path, package_dir, &named.dir);
    }
    if explicit_external(specifier) {
        return false;
    }
    package_version(&named.dir, sources)
        .is_some_and(|version| version_satisfies(specifier, &version))
}

fn workspace_protocol_edge(rest: &str, named: &WorkspacePackage, package_dir: &Path) -> bool {
    match relative_protocol_path(rest) {
        Some(path) => path_is_package_dir(path, package_dir, &named.dir),
        None => true,
    }
}

fn local_protocol_path(specifier: &str) -> Option<&str> {
    ["file:", "link:", "portal:"]
        .into_iter()
        .find_map(|prefix| specifier.strip_prefix(prefix))
        .and_then(relative_protocol_path)
}

fn relative_protocol_path(path: &str) -> Option<&str> {
    let path = path.trim();
    (path.starts_with('.') && !path.contains("://") && !archive_path(path)).then_some(path)
}

fn archive_path(path: &str) -> bool {
    path.ends_with(".tgz") || path.ends_with(".tar") || path.ends_with(".tar.gz")
}

fn path_is_package_dir(path: &str, from_dir: &Path, package_dir: &Path) -> bool {
    let raw = path.trim_end_matches(['/', '\\']);
    if raw.is_empty() {
        return false;
    }
    let joined = Path::new(raw);
    let target = if joined.is_absolute() {
        normalize_path(joined)
    } else {
        normalize_path(&from_dir.join(raw))
    };
    let target = package_directory(target);
    normalize_path(package_dir) == target
}

fn package_directory(target: PathBuf) -> PathBuf {
    match target.file_name().and_then(|name| name.to_str()) {
        Some("package.json") => target.parent().map(normalize_path).unwrap_or(target),
        _ => target,
    }
}

fn explicit_external(specifier: &str) -> bool {
    [
        "catalog:",
        "npm:",
        "jsr:",
        "git:",
        "git+",
        "github:",
        "http:",
        "https:",
        "gist:",
        "bitbucket:",
        "gitlab:",
    ]
    .iter()
    .any(|prefix| specifier.starts_with(prefix))
}

fn package_version(
    package_dir: &Path,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Option<String> {
    let manifest = sources
        .parse_json_path(&package_dir.join("package.json"))
        .ok()?;
    let version = manifest.get("version")?.as_str()?.trim();
    (!version.is_empty()).then(|| version.to_string())
}
