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

use crate::codebase::dependencies::graph::{
    DepGraph, GraphBuildPlan, GraphFiles, PreparedGraphBuild,
};
use crate::codebase::dependencies::{EdgeKind, NodeId};
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::relative_slash_path;

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
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current());
    let dataset = session.dataset(&root);
    let visible_paths = dataset.visible_paths_arc();
    let root_visible_paths = dataset.paths_for(&root);
    let sources = dataset.sources_for(&root);
    let mut graph_files = GraphFiles::from_files_with_resource_candidates(
        crate::codebase::ts_source::discover_files_from_visible(&root, &[], &root_visible_paths),
        visible_paths.tracked_paths_for(&root).as_ref().clone(),
    );
    let config = dataset.config(config_path)?;
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
    let explicit_tsconfig = tsconfig;
    let (tsconfig, tsconfig_catalog) = match explicit_tsconfig {
        None => {
            let workspace = dataset.workspace();
            let catalog =
                crate::codebase::ts_resolver::TsConfigCatalog::from_visible_and_sources_with_workspace(
                &root,
                std::slice::from_ref(&root),
                &root_visible_paths,
                &sources,
                &workspace,
            );
            let config = catalog.config_for(&entry_abs).clone();
            (config, catalog)
        }
        Some(path) => {
            let config = (*dataset.tsconfig(Some(path))?).clone();
            let path = if path.is_absolute() {
                path.to_path_buf()
            } else {
                root.join(path)
            };
            let catalog = crate::codebase::ts_resolver::TsConfigCatalog::forced(
                &root,
                config.clone(),
                Some(normalize_path(&path)),
            );
            (config, catalog)
        }
    };
    let allowed = runtime_edges();
    // Build only the runtime-import edges we traverse, not every edge producer
    // (routes, queues, React, Swift, …), which an `effects` query discards.
    let plan = GraphBuildPlan::from_allowed(Some(&allowed));
    let mut fact_context = crate::codebase::ts_source::facts::TsFactContext::new(&root);
    fact_context.effect_functions = selection.names.clone();
    fact_context.set_visible_files(graph_files.visible().iter().cloned());
    let facts =
        crate::codebase::ts_source::facts::collect_ts_facts_with_context_sources_and_session(
            &session,
            graph_files.indexable(),
            crate::codebase::ts_source::facts::TsFactPlan {
                imports: true,
                function_calls: true,
                effect_calls: true,
                ..Default::default()
            },
            &fact_context,
            &sources,
        );
    crate::invocation::check_timeout()?;
    let codebase_config =
        crate::codebase::config::config_from_loaded_v2(&root, config_path, &config);
    let prepared_graph = crate::codebase::dependencies::graph::prepare_graph_config(
        &root,
        plan,
        &codebase_config,
        &config,
        &visible_paths,
    )?;
    let interner = session.interner_arc();
    let graph = DepGraph::build_with_plan_files_prepared_config_facts_resolution_cache_and_session(
        PreparedGraphBuild {
            root: &root,
            tsconfig: &tsconfig,
            tsconfig_catalog: Some(&tsconfig_catalog),
            plan,
            graph_files: &graph_files,
            config_path,
            prepared: &prepared_graph,
            facts: Some(&facts),
            import_resolution_cache: None,
            dotnet_facts: None,
            swift_facts: None,
            visible_paths: Some(&visible_paths),
        },
        session,
    )?;

    run_with_prepared(&root, &selection, entry, depth, &graph, &facts, &interner)
}

include!("effects_query/prepared.rs");

#[cfg(test)]
mod tests;
