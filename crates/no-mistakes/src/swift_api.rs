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
use crate::config::v2::load_v2_config_from_visible;

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
    )?;
    let all_files = graph_files.all();
    let facts = collect_swift_facts(&root, all_files, &packages);
    let graph = DepGraph::build_with_plan_files_prepared_config_and_swift_facts(
        &root,
        &tsconfig,
        plan,
        &graph_files,
        config_path,
        &prepared_graph,
        &facts,
    )?;
    let mut targets = HashMap::new();
    for package in &facts.packages {
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
        let node = NodeId::File(path);
        let entries = self.graph.dependents_of(&[node], None, Some(&allowed));
        for path in entries.iter().filter_map(|entry| entry.node.as_file()) {
            self.record_test_target(path, &mut seen);
        }
        seen.into_iter()
            .map(|(target, package)| TestTargetRow {
                command: test_command(&package, &target),
                target,
                package,
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

fn test_command(package: &str, target: &str) -> String {
    // `swift test --filter` matches a regex against `<target>.<case>`; anchor to
    // the target prefix and escape it so one target name cannot match another.
    // Single-quote it so the shell preserves the regex backslashes verbatim.
    let filter = shell_single_quote(&format!("^{}\\.", regex_escape(target)));
    if package.is_empty() || package == "." {
        format!("swift test --filter {filter}")
    } else {
        format!(
            "swift test --package-path {} --filter {filter}",
            shell_single_quote(package)
        )
    }
}

fn regex_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        if "\\.+*?()|[]{}^$".contains(ch) {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    escaped
}

/// Wrap a value in single quotes for POSIX shells, escaping embedded quotes so a
/// package path like `bob's app` stays a single argument.
fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests;
