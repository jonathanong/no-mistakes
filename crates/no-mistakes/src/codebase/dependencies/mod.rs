pub mod extract;
pub mod graph;
pub mod output;
pub mod query;

use anyhow::{bail, Context, Result};
use globset::{Glob, GlobSetBuilder};
use is_terminal::IsTerminal;
use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::codebase::analysis_session::PathInterner;
use crate::codebase::testing_analysis::{self, PreparedProjects};
pub use crate::codebase::ts_resolver::TsConfig;
pub use graph::{DepGraph, EdgeKind, NodeId};
pub(crate) use testing_analysis::FrameworkCatalogPreparation;

pub use crate::cli::Format;

include!("args_test_globs.rs");
include!("args_relationships.rs");
include!("projection.rs");
include!("candidate_inventory.rs");

include!("traversal_entrypoints.rs");
include!("traversal_validation.rs");
include!("traversal_queue_roots.rs");
include!("symbol_resolution.rs");
include!("shared_traversal.rs");
include!("shared_traversal_bounded.rs");
include!("shared_traversal_bounded_seed.rs");
include!("shared_traversal_prepare.rs");
include!("shared_traversal_facts.rs");
include!("shared_traversal_lazy_graph.rs");
include!("shared_traversal_reports.rs");
include!("output_results.rs");
include!("shared_graph_cache.rs");
include!("shared_traversal_graph.rs");
include!("shared_traversal_graph_shared.rs");
include!("shared_traversal_collect.rs");
include!("shared_traversal_collect_diagnostics.rs");
include!("shared_traversal_collect_cache.rs");
include!("shared_traversal_collect_roots.rs");
include!("shared_traversal_provenance.rs");
include!("shared_traversal_uncached.rs");
include!("shared_traversal_uncached_helpers.rs");
include!("output_args.rs");
include!("collect_standalone.rs");
include!("run.rs");

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod shared_traversal_collect_cache_tests;
#[cfg(test)]
mod shared_traversal_collect_diagnostics_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod shared_traversal_facts_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod shared_traversal_lazy_graph_tests;
#[cfg(any(test, feature = "test-support"))]
mod shared_traversal_test_api;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod traversal;

#[cfg(test)]
mod root_dependency_test_helpers;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod traversal_entrypoint_test_helpers;

pub struct TraversalResult {
    entries: Vec<graph::NodeEntry>,
    root: PathBuf,
    diagnostics: Vec<crate::codebase::ts_resolver::TsConfigDiagnostic>,
    tsconfig_provenance: Vec<crate::codebase::ts_resolver::TsConfigProvenance>,
    projection: TraverseProjection,
}

impl TraversalResult {
    /// File nodes in a traversal result. Derived reports use this projection
    /// instead of re-running source discovery or graph traversal.
    pub fn file_paths(&self) -> impl Iterator<Item = &std::path::Path> {
        self.entries.iter().filter_map(|entry| entry.node.as_file())
    }
}

fn apply_filters(
    entries: Vec<graph::NodeEntry>,
    args: &TraverseArgs,
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    tsconfig: &TsConfig,
    visible_paths: &crate::codebase::ts_source::VisiblePathSnapshot,
    prepared_test_projects: Option<&dyn PreparedProjects>,
) -> Result<Vec<graph::NodeEntry>> {
    // Build combined filter from --filter and --test globs.
    let mut all_filters = args.filters.clone();
    for framework in &args.tests {
        all_filters.extend(test_filters_from_prepared(
            root,
            framework,
            config,
            tsconfig,
            visible_paths,
            prepared_test_projects,
        ));
    }
    let filter = graph::build_filter(&all_filters)?;
    let entries = graph::apply_filter(entries, filter.as_ref(), root);
    let entries = if !all_filters.is_empty() && args.target_modules.is_empty() {
        entries
            .into_iter()
            .filter(|entry| !matches!(entry.node, graph::NodeId::Module(_)))
            .collect()
    } else {
        entries
    };
    apply_target_module_filters(entries, &args.target_modules)
}

fn test_filters_from_prepared(
    root: &Path,
    framework: &str,
    config: &crate::config::v2::NoMistakesConfig,
    tsconfig: &TsConfig,
    visible_paths: &crate::codebase::ts_source::VisiblePathSnapshot,
    prepared_test_projects: Option<&dyn PreparedProjects>,
) -> Vec<String> {
    (testing_analysis::hooks().test_filters_from_prepared)(
        root,
        framework,
        config,
        tsconfig,
        visible_paths,
        prepared_test_projects,
    )
}

pub(crate) fn prepare_tsconfig_catalog_with_framework_projects(
    input: FrameworkCatalogPreparation<'_>,
) -> Result<testing_analysis::FrameworkCatalog> {
    (testing_analysis::hooks().prepare_framework_catalog)(input)
}

fn apply_target_module_filters(
    entries: Vec<graph::NodeEntry>,
    target_modules: &[String],
) -> Result<Vec<graph::NodeEntry>> {
    if target_modules.is_empty() {
        return Ok(entries);
    }
    let mut builder = GlobSetBuilder::new();
    for pattern in target_modules {
        builder.add(Glob::new(pattern)?);
    }
    let filter = builder.build()?;
    Ok(entries
        .into_iter()
        .filter(|entry| match &entry.node {
            graph::NodeId::Module(specifier) => filter.is_match(specifier.as_ref()),
            _ => false,
        })
        .collect())
}
