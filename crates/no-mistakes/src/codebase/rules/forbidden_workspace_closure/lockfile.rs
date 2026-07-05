use super::{Dependency, PackageNode, RULE_ID};
use crate::codebase::ts_source::relative_slash_path;
use crate::codebase::workspaces;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub(super) fn lockfile_nodes(
    root: &Path,
    lockfile: &Path,
    workspace: &workspaces::WorkspaceMap,
    manifest_nodes: &BTreeMap<String, PackageNode>,
    dependency_types: &[&str],
) -> std::result::Result<BTreeMap<String, PackageNode>, String> {
    if lockfile.file_name().and_then(|name| name.to_str()) != Some("pnpm-lock.yaml") {
        return Err(format!(
            "{RULE_ID}: lockfile currently supports pnpm-lock.yaml only"
        ));
    }
    let dependency_types = validate_dependency_types(dependency_types)?;
    let lockfile_path = absolute_lockfile_path(root, lockfile);
    let lockfile_root = lockfile_path.parent().unwrap_or(root);
    let content = std::fs::read_to_string(&lockfile_path).map_err(|error| {
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
    let mut nodes = BTreeMap::new();
    for package in &workspace.packages {
        let rel_dir = relative_slash_path(lockfile_root, &package.dir);
        let importer_key = if rel_dir.is_empty() {
            ".".to_string()
        } else {
            rel_dir
        };
        let Some(importer) = importer_by_path.get(&importer_key) else {
            return Err(format!(
                "{RULE_ID}: lockfile is missing importer for workspace package '{}'",
                package.name
            ));
        };
        let manifest = package.dir.join("package.json");
        let deps =
            lockfile_dependencies(importer, &manifest_nodes[&package.name], &dependency_types);
        nodes.insert(package.name.clone(), PackageNode { manifest, deps });
    }
    Ok(nodes)
}

#[derive(Debug, Clone, Copy)]
enum LockfileDependencyType {
    Dependencies,
    DevDependencies,
    PeerDependencies,
    OptionalDependencies,
}

impl LockfileDependencyType {
    fn field(self) -> &'static str {
        match self {
            Self::Dependencies => "dependencies",
            Self::DevDependencies => "devDependencies",
            Self::PeerDependencies => "peerDependencies",
            Self::OptionalDependencies => "optionalDependencies",
        }
    }

    fn importer_entries(
        self,
        importer: &crate::codebase::lockfile::pnpm::PnpmImporter,
    ) -> Option<(
        &'static str,
        &[crate::codebase::lockfile::pnpm::PnpmImporterDependency],
    )> {
        match self {
            Self::Dependencies => Some((self.field(), &importer.dependencies)),
            Self::DevDependencies => Some((self.field(), &importer.dev_dependencies)),
            Self::PeerDependencies => None,
            Self::OptionalDependencies => Some((self.field(), &importer.optional_dependencies)),
        }
    }
}

fn validate_dependency_types(
    dependency_types: &[&str],
) -> std::result::Result<Vec<LockfileDependencyType>, String> {
    let mut validated = Vec::new();
    for field in dependency_types {
        validated.push(match *field {
            "dependencies" => LockfileDependencyType::Dependencies,
            "devDependencies" => LockfileDependencyType::DevDependencies,
            "peerDependencies" => LockfileDependencyType::PeerDependencies,
            "optionalDependencies" => LockfileDependencyType::OptionalDependencies,
            _ => {
                return Err(format!(
                    "{RULE_ID}: lockfile dependencyTypes supports dependencies, devDependencies, peerDependencies, and optionalDependencies only; unsupported dependency type '{field}'"
                ));
            }
        });
    }
    Ok(validated)
}

fn absolute_lockfile_path(root: &Path, lockfile: &Path) -> PathBuf {
    if lockfile.is_absolute() {
        lockfile.to_path_buf()
    } else {
        root.join(lockfile)
    }
}

fn lockfile_dependencies(
    importer: &crate::codebase::lockfile::pnpm::PnpmImporter,
    manifest_node: &PackageNode,
    dependency_types: &[LockfileDependencyType],
) -> Vec<Dependency> {
    let mut deps = Vec::new();
    for field in dependency_types {
        if let Some((field_name, entries)) = field.importer_entries(importer) {
            deps.extend(entries.iter().map(|entry| Dependency {
                name: entry.alias.clone(),
                resolved_name: entry.resolution_name.clone(),
                field: field_name.to_string(),
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
