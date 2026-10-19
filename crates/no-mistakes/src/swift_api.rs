//! Library entrypoint for the `swift` command and its N-API parity bindings.
//!
//! Re-exposes the already-integrated Swift graph facts as two queries:
//! `importers` (reverse `SwiftImport`/`SwiftReference` traversal) and
//! `test-targets` (reverse traversal restricted to files in SwiftPM test targets).

use anyhow::Result;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::codebase::dependencies::graph::{DepGraph, EdgeKind, GraphBuildPlan, NodeId};
use crate::codebase::swift::collect_swift_facts;
use crate::codebase::ts_resolver::{normalize_path, TsConfig};
use crate::config::v2::load_v2_config_from_visible;
use crate::config::v2::schema::SwiftConfig;

mod command;
use command::test_command;

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
    /// Cloned `tests.swift.baseCommands`. Missing packages stay `swift test`.
    base_commands: BTreeMap<String, Vec<String>>,
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
    /// Command that runs just this target. A declared package uses
    /// `tests.swift.baseCommands`; otherwise this is `swift test`.
    pub command: String,
}

/// Build the Swift graph and target index once.
pub fn analyze_project(root: &Path, config_path: Option<&Path>) -> Result<SwiftReport> {
    crate::invocation::check_timeout()?;
    let root = normalize_path(root);
    let visible_paths = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let root_visible_paths = visible_paths.paths_for(&root);
    // Propagate errors so an explicit but missing/invalid `--config` is reported
    // instead of silently producing an empty result.
    let config = load_v2_config_from_visible(&root, config_path, &root_visible_paths)?;
    let packages = config.tests.swift.packages.clone();

    let tsconfig = TsConfig::default();
    let plan = GraphBuildPlan {
        swift: true,
        ..GraphBuildPlan::default()
    };
    let graph_files = crate::codebase::dependencies::graph::GraphFiles::from_files(
        crate::codebase::ts_source::discover_files_from_visible(&root, &[], &root_visible_paths),
    );
    let codebase_config =
        crate::codebase::config::config_from_loaded_v2(&root, config_path, &config);
    let prepared_graph = crate::codebase::dependencies::graph::prepare_graph_config(
        &root,
        plan,
        &codebase_config,
        &config,
        &visible_paths,
    );
    let prepared_graph = prepared_graph?;
    let all_files = graph_files.all();
    let facts = collect_swift_facts(&root, all_files, &packages);
    crate::invocation::check_timeout()?;
    let graph = DepGraph::build_with_plan_files_prepared_config_and_swift_facts(
        &root,
        &tsconfig,
        plan,
        &graph_files,
        config_path,
        &prepared_graph,
        &facts,
    );
    let graph = graph?;
    crate::invocation::check_timeout()?;
    let mut targets = HashMap::new();
    for package in &facts.packages {
        crate::invocation::check_timeout()?;
        for (name, target) in &package.targets {
            // Target names are unique only within a package, and nested packages
            // can share a name, so scope membership to this target's own source
            // roots rather than the (prefix-matching) package root.
            let files = facts.files_by_target.get(name).into_iter().flatten();
            for file in files.filter(|path| target.roots.iter().any(|r| path.starts_with(r))) {
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

    crate::invocation::check_timeout()?;
    Ok(SwiftReport {
        root,
        graph,
        targets,
        base_commands: config.tests.swift.base_commands.clone(),
    })
}

impl SwiftReport {
    /// Swift files that import or reference the given file (direct + transitive).
    pub fn importers(&self, file: &str) -> Vec<ImporterRow> {
        let node = NodeId::file(normalize_path(&self.root.join(file)));
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
        let path = normalize_path(&self.root.join(file));
        let allowed: HashSet<EdgeKind> = [
            EdgeKind::SwiftImport,
            EdgeKind::SwiftReference,
            EdgeKind::SwiftPackageDependency,
        ]
        .into();
        let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
        // The queried file's own test target covers it, but `dependents_of` does
        // not return the root node, so seed it explicitly.
        self.record_test_target(&path, &mut seen);
        let node = NodeId::file(path);
        let entries = self.graph.dependents_of(&[node], None, Some(&allowed));
        for path in entries.iter().filter_map(|entry| entry.node.as_file()) {
            self.record_test_target(path, &mut seen);
        }
        seen.into_iter()
            .map(|(target, package)| {
                let base = SwiftConfig::base_command_in(&self.base_commands, &package);
                TestTargetRow {
                    command: test_command(&package, &target, base),
                    target,
                    package,
                }
            })
            .collect()
    }

    fn record_test_target(&self, path: &Path, seen: &mut BTreeSet<(String, String)>) {
        if let Some(info) = self.targets.get(path) {
            if info.is_test {
                seen.insert((info.target.clone(), self.rel(&info.package_root)));
            }
        }
    }

    fn rel(&self, path: &Path) -> String {
        path.strip_prefix(&self.root)
            .unwrap_or(path)
            .display()
            .to_string()
    }
}

#[cfg(test)]
mod tests;
