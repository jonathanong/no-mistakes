//! Library entrypoint for the `swift` command and its N-API parity bindings.
//!
//! Re-exposes the already-integrated Swift graph facts as two queries:
//! `importers` (reverse `SwiftImport`/`SwiftReference` traversal) and
//! `test-targets` (reverse traversal restricted to files in SwiftPM test targets).

use anyhow::Result;
use serde::Serialize;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::codebase::dependencies::graph::{DepGraph, EdgeKind, GraphBuildPlan, NodeId};
use crate::codebase::swift::collect_swift_facts;
use crate::codebase::ts_resolver::{normalize_path, TsConfig};
use crate::config::v2::load_v2_config;

struct TargetInfo {
    target: String,
    is_test: bool,
    package_root: PathBuf,
}

/// A built Swift graph plus the file→target index used to answer queries.
pub struct SwiftReport {
    root: PathBuf,
    graph: DepGraph,
    targets: HashMap<PathBuf, TargetInfo>,
}

/// One Swift file that imports/references the queried file.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ImporterRow {
    /// The importing file, relative to the repo root.
    pub file: String,
    /// Traversal depth (1 = direct importer).
    pub depth: usize,
}

/// One SwiftPM test target that transitively covers the queried file.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct TestTargetRow {
    pub target: String,
    /// The SwiftPM package directory, relative to the repo root.
    pub package: String,
    /// The `swift test` command that runs just this target.
    pub command: String,
}

/// Build the Swift graph and target index once.
pub fn analyze_project(root: &Path, config_path: Option<&Path>) -> Result<SwiftReport> {
    let root = normalize_path(root);
    // Propagate errors so an explicit but missing/invalid `--config` is reported
    // instead of silently producing an empty result.
    let config = load_v2_config(&root, config_path)?;
    let packages = config.tests.swift.packages.clone();

    let tsconfig = TsConfig::default();
    let plan = GraphBuildPlan {
        swift: true,
        ..GraphBuildPlan::default()
    };
    let graph = DepGraph::build_with_plan_and_config(&root, &tsconfig, plan, config_path)?;

    let all_files = crate::codebase::ts_source::discover_files(&root, &[]);
    let facts = collect_swift_facts(&root, &all_files, &packages);
    let mut targets = HashMap::new();
    for package in &facts.packages {
        // Target names are unique only within a package, so scope membership to
        // files under this package's root before attaching its metadata.
        let files_for = |name: &String| {
            facts
                .files_by_target
                .get(name)
                .into_iter()
                .flatten()
                .filter(|path| path.starts_with(&package.package_root))
        };
        for (name, target) in &package.targets {
            for file in files_for(name) {
                targets.insert(
                    file.clone(),
                    TargetInfo {
                        target: name.clone(),
                        is_test: target.is_test,
                        package_root: package.package_root.clone(),
                    },
                );
            }
        }
    }

    Ok(SwiftReport {
        root,
        graph,
        targets,
    })
}

impl SwiftReport {
    /// Swift files that import or reference the given file (direct + transitive).
    pub fn importers(&self, file: &str) -> Vec<ImporterRow> {
        let node = NodeId::File(normalize_path(&self.root.join(file)));
        let allowed: HashSet<EdgeKind> = [EdgeKind::SwiftImport, EdgeKind::SwiftReference].into();
        let mut rows: Vec<ImporterRow> = self
            .graph
            .dependents_of(&[node], None, Some(&allowed))
            .into_iter()
            .filter_map(|entry| {
                entry.node.as_file().map(|path| ImporterRow {
                    file: self.rel(path),
                    depth: entry.depth,
                })
            })
            .collect();
        rows.sort();
        rows.dedup();
        rows
    }

    /// SwiftPM test targets that transitively cover the given file.
    pub fn test_targets(&self, file: &str) -> Vec<TestTargetRow> {
        let node = NodeId::File(normalize_path(&self.root.join(file)));
        let allowed: HashSet<EdgeKind> = [
            EdgeKind::SwiftImport,
            EdgeKind::SwiftReference,
            EdgeKind::SwiftPackageDependency,
        ]
        .into();
        let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
        let entries = self.graph.dependents_of(&[node], None, Some(&allowed));
        for path in entries.iter().filter_map(|entry| entry.node.as_file()) {
            if let Some(info) = self.targets.get(path) {
                if info.is_test {
                    seen.insert((info.target.clone(), self.rel(&info.package_root)));
                }
            }
        }
        seen.into_iter()
            .map(|(target, package)| TestTargetRow {
                command: test_command(&package, &target),
                target,
                package,
            })
            .collect()
    }

    fn rel(&self, path: &Path) -> String {
        path.strip_prefix(&self.root)
            .unwrap_or(path)
            .display()
            .to_string()
    }
}

fn test_command(package: &str, target: &str) -> String {
    if package.is_empty() || package == "." {
        format!("swift test --filter {target}")
    } else {
        format!("swift test --package-path {package} --filter {target}")
    }
}

#[cfg(test)]
mod tests;
