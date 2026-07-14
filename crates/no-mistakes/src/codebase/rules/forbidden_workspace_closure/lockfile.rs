use super::{Dependency, PackageNode, RULE_ID};
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::relative_slash_path;
use crate::codebase::workspaces;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

mod dependency_type;
mod path_alias;

pub(super) fn base_root<'a>(root: &'a Path, target_roots: &'a [PathBuf]) -> &'a Path {
    target_roots
        .first()
        .filter(|_| target_roots.len() == 1)
        .map_or(root, PathBuf::as_path)
}

pub(super) fn lockfile_nodes(
    root: &Path,
    lockfile_base: &Path,
    lockfile: &Path,
    workspace: &workspaces::WorkspaceMap,
    manifest_nodes: &BTreeMap<String, PackageNode>,
    request: (&[&str], &[String], &crate::codebase::ts_source::SourceStore),
) -> std::result::Result<BTreeMap<String, PackageNode>, String> {
    let (dependency_types, start_packages, sources) = request;
    if lockfile.file_name().and_then(|name| name.to_str()) != Some("pnpm-lock.yaml") {
        return Err(format!(
            "{RULE_ID}: lockfile currently supports pnpm-lock.yaml only"
        ));
    }
    let dependency_types = dependency_type::validate(dependency_types)?;
    let lockfile_path = absolute_lockfile_path(root, lockfile_base, lockfile);
    let lockfile_root = lockfile_path.parent().unwrap_or(root);
    let content = sources.read_path(&lockfile_path).map_err(|error| {
        format!(
            "{RULE_ID}: could not read lockfile {}: {error}",
            relative_slash_path(root, &lockfile_path)
        )
    })?;
    let importers = crate::codebase::lockfile::pnpm::parse_importers(&content);
    if importers.is_empty() {
        return Err(format!(
            "{RULE_ID}: lockfile {} has no pnpm importers",
            relative_slash_path(root, &lockfile_path)
        ));
    }
    let importer_by_path: BTreeMap<String, _> = importers
        .into_iter()
        .map(|importer| (normalize_importer_path(&importer.path), importer))
        .collect();
    let package_by_name: BTreeMap<String, _> = workspace
        .packages
        .iter()
        .map(|package| (package.name.clone(), package))
        .collect();
    let workspace_names: BTreeSet<String> = package_by_name.keys().cloned().collect();
    let package_by_dir: BTreeMap<PathBuf, String> = workspace
        .packages
        .iter()
        .map(|package| (package.dir.clone(), package.name.clone()))
        .collect();
    let mut nodes = BTreeMap::new();
    let mut queued = BTreeSet::new();
    let mut queue = VecDeque::new();
    for package in start_packages {
        if manifest_nodes.contains_key(package) && queued.insert(package.clone()) {
            queue.push_back(package.clone());
        }
    }
    while let Some(package_name) = queue.pop_front() {
        let package = package_by_name[&package_name];
        let importer_key = importer_key(lockfile_root, &package.dir);
        let Some(importer) = importer_by_path.get(&importer_key) else {
            return Err(format!(
                "{RULE_ID}: lockfile is missing importer for workspace package '{}'",
                package.name
            ));
        };
        let manifest = package.dir.join("package.json");
        let deps = lockfile_dependencies(
            lockfile_root,
            importer,
            &package_by_dir,
            &workspace_names,
            &manifest_nodes[&package_name],
            &dependency_types,
        );
        for dep in &deps {
            let Some(workspace_dep) = dep.workspace_name.as_ref() else {
                continue;
            };
            if workspace_names.contains(workspace_dep) && queued.insert(workspace_dep.clone()) {
                queue.push_back(workspace_dep.clone());
            }
        }
        nodes.insert(package_name, PackageNode { manifest, deps });
    }
    Ok(nodes)
}

fn importer_key(lockfile_root: &Path, package_dir: &Path) -> String {
    let rel_dir = relative_slash_path(lockfile_root, package_dir);
    if rel_dir.is_empty() {
        ".".to_string()
    } else {
        rel_dir
    }
}

fn absolute_lockfile_path(root: &Path, lockfile_base: &Path, lockfile: &Path) -> PathBuf {
    let path = if lockfile.is_absolute() {
        lockfile.to_path_buf()
    } else if lockfile_base != root {
        let project_lockfile = lockfile_base.join(lockfile);
        if project_lockfile.exists() {
            project_lockfile
        } else {
            root.join(lockfile)
        }
    } else {
        root.join(lockfile)
    };
    normalize_path(&path)
}

fn lockfile_dependencies(
    lockfile_root: &Path,
    importer: &crate::codebase::lockfile::pnpm::PnpmImporter,
    package_by_dir: &BTreeMap<PathBuf, String>,
    workspace_names: &BTreeSet<String>,
    manifest_node: &PackageNode,
    dependency_types: &[dependency_type::LockfileDependencyType],
) -> Vec<Dependency> {
    let mut deps = Vec::new();
    for field in dependency_types {
        if let Some((field_name, entries)) = field.importer_entries(importer) {
            deps.extend(entries.iter().filter_map(|entry| {
                let manifest_field = manifest_node
                    .deps
                    .iter()
                    .find(|dep| dep.name == entry.alias)
                    .map(|dep| dep.field.as_str());
                if manifest_field.is_some_and(|manifest_field| manifest_field != field_name) {
                    return None;
                }
                let path_workspace_name = path_alias::resolve_workspace_path_dependency(
                    lockfile_root,
                    importer,
                    entry,
                    package_by_dir,
                );
                let workspace_name = if let Some(name) = path_workspace_name.clone() {
                    Some(name)
                } else if entry.specifier.starts_with("workspace:")
                    && workspace_names.contains(&entry.alias)
                {
                    Some(entry.alias.clone())
                } else {
                    None
                };
                Some(Dependency {
                    name: entry.alias.clone(),
                    resolved_name: entry.resolution_name.clone().or(path_workspace_name),
                    workspace_name,
                    field: manifest_field.unwrap_or(field_name).to_string(),
                })
            }));
        } else {
            deps.extend(
                manifest_node
                    .deps
                    .iter()
                    .filter(|dep| dep.field == field.field())
                    .cloned(),
            );
        }
    }
    deps.sort_by(|a, b| {
        a.name
            .cmp(&b.name)
            .then(a.resolved_name.cmp(&b.resolved_name))
            .then(a.field.cmp(&b.field))
    });
    deps.dedup();
    deps
}

pub(super) fn normalize_importer_path(path: &str) -> String {
    let normalized = path.trim_start_matches("./").trim_end_matches('/');
    if normalized.is_empty() {
        ".".to_string()
    } else {
        normalized.to_string()
    }
}
