use super::{Dependency, PackageNode, RULE_ID};
use crate::codebase::ts_source::relative_slash_path;
use crate::codebase::workspaces;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

mod dependency_type;
mod importer_cache;
mod path_alias;
mod paths;
pub(super) use importer_cache::LockfileImporterCache;
use importer_cache::ParsedImporters;
use paths::{absolute_lockfile_path, importer_key};
pub(super) use paths::{base_root, normalize_importer_path};

pub(super) fn lockfile_nodes(
    root: &Path,
    lockfile_base: &Path,
    lockfile: &Path,
    workspace: &workspaces::WorkspaceMap,
    manifest_nodes: &BTreeMap<String, PackageNode>,
    importer_cache: &LockfileImporterCache,
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
    let rel_lockfile = relative_slash_path(root, &lockfile_path);
    let parsed = importer_cache.get_or_compute(&lockfile_path, || {
        let content = sources
            .read_path(&lockfile_path)
            .map_err(|error| error.to_string())?;
        importer_cache::parse_importer_content(&content)
    });
    let parsed = parsed.as_ref().as_ref().map_err(|error| {
        format!(
            "{RULE_ID}: could not read lockfile {}: {error}",
            relative_slash_path(root, &lockfile_path)
        )
    })?;
    let importers = match parsed {
        ParsedImporters::Malformed => {
            return Err(format!(
                "{RULE_ID}: lockfile {rel_lockfile} could not be parsed"
            ));
        }
        ParsedImporters::Loaded {
            env_prefix_valid: false,
            ..
        } => {
            return Err(format!(
                "{RULE_ID}: lockfile {rel_lockfile} has a non-env document before the project lockfile"
            ));
        }
        ParsedImporters::Loaded { by_path, .. } => by_path,
    };
    if importers.is_empty() {
        return Err(format!(
            "{RULE_ID}: lockfile {} has no pnpm importers",
            relative_slash_path(root, &lockfile_path)
        ));
    }
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
        let Some(importer) = importers.get(&importer_key) else {
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
