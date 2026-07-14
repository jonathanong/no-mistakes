//! `effects <kind> --entry <file>`: report every transitive call site of a
//! configured set of effect functions/constructors that is reachable from
//! `<entry>` through the import graph.
//!
//! The function/constructor names per `<kind>` come entirely from configuration
//! (`effects.<kind>` in `.no-mistakes.yml`); nothing is hardcoded. Reachability
//! reuses the canonical dependency graph ([`DepGraph::deps_of`]) over runtime
//! import edges, then each reachable file is parsed once to collect matching
//! call sites with line numbers.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use serde::Serialize;

use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan, GraphFiles};
use crate::codebase::dependencies::{EdgeKind, NodeId};
use crate::codebase::ts_resolver::{
    find_tsconfig_from_visible, load_tsconfig, normalize_path, TsConfig,
};
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::load_v2_config_from_visible;

/// One matched effect call site.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct EffectCallSite {
    pub file: String,
    pub line: usize,
    pub callee: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<String>,
    pub depth: usize,
}

/// The full `effects <kind>` report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectsReport {
    pub kind: String,
    pub entry: String,
    pub call_sites: Vec<EffectCallSite>,
    pub by_category: BTreeMap<String, usize>,
}

pub(crate) struct EffectsSelection {
    kind: String,
    names: HashMap<String, Option<String>>,
}

impl EffectsReport {
    /// Sorted unique matched file paths, for `--format paths`.
    pub fn paths(&self) -> Vec<String> {
        let mut paths: Vec<String> = self
            .call_sites
            .iter()
            .map(|site| site.file.clone())
            .collect();
        paths.sort();
        paths.dedup();
        paths
    }
}

/// Edge kinds that represent runtime reachability (code that actually executes
/// when the entry module is imported). Type-only imports are excluded.
fn runtime_edges() -> HashSet<EdgeKind> {
    HashSet::from([
        EdgeKind::Import,
        EdgeKind::DynamicImport,
        EdgeKind::Require,
        // Workspace-package imports are runtime imports in a monorepo.
        EdgeKind::WorkspaceImport,
    ])
}

fn resolve_tsconfig_from_visible(
    root: &Path,
    tsconfig: Option<&Path>,
    visible_paths: &[PathBuf],
) -> Result<TsConfig> {
    match tsconfig {
        // Resolve a relative explicit tsconfig against `root`, not the cwd.
        Some(path) if path.is_absolute() => load_tsconfig(path),
        Some(path) => load_tsconfig(&root.join(path)),
        None => match find_tsconfig_from_visible(root, visible_paths) {
            Some(path) => load_tsconfig(&path),
            None => Ok(TsConfig {
                dir: root.to_path_buf(),
                paths: vec![],
                paths_dir: root.to_path_buf(),
                base_url: None,
            }),
        },
    }
}

/// Run the `effects <kind>` query.
pub fn run(
    root: &Path,
    config_path: Option<&Path>,
    tsconfig: Option<&Path>,
    kind: &str,
    entry: &Path,
    categories: &[String],
    depth: Option<usize>,
) -> Result<EffectsReport> {
    let root = normalize_path(root);
    let root = root.canonicalize().unwrap_or(root);
    let visible_paths = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let root_visible_paths = visible_paths.paths_for(&root);
    let mut graph_files = GraphFiles::from_files(
        crate::codebase::ts_source::discover_files_from_visible(&root, &[], &root_visible_paths),
    );
    let config = load_v2_config_from_visible(&root, config_path, &root_visible_paths)?;
    let selection = selection_from_config(&config, kind, categories)?;

    let entry_abs = if entry.is_absolute() {
        entry.to_path_buf()
    } else {
        root.join(entry)
    };
    if !entry_abs.is_file() {
        bail!("entry file not found: {}", entry_abs.display());
    }
    graph_files.add_explicit_root(&entry_abs);
    let tsconfig = resolve_tsconfig_from_visible(&root, tsconfig, graph_files.all())?;
    let allowed = runtime_edges();
    // Build only the runtime-import edges we traverse, not every edge producer
    // (routes, queues, React, Swift, …), which an `effects` query discards.
    let plan = GraphBuildPlan::from_allowed(Some(&allowed));
    let mut fact_context = crate::codebase::ts_source::facts::TsFactContext::new(&root);
    fact_context.effect_functions = selection.names.clone();
    fact_context.set_visible_files(graph_files.visible().iter().cloned());
    let facts = crate::codebase::ts_source::facts::collect_ts_facts_with_context(
        graph_files.indexable(),
        crate::codebase::ts_source::facts::TsFactPlan {
            imports: true,
            function_calls: true,
            effect_calls: true,
            ..Default::default()
        },
        &fact_context,
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
    let graph = DepGraph::build_with_plan_files_prepared_config_and_facts(
        &root,
        &tsconfig,
        plan,
        &graph_files,
        config_path,
        &prepared_graph,
        Some(&facts),
    )?;

    run_with_prepared(&root, &selection, entry, depth, &graph, &facts)
}

include!("effects_query/prepared.rs");

#[cfg(test)]
mod tests;
