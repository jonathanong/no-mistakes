pub mod extract;
pub mod graph;
pub mod output;

use anyhow::{bail, Context, Result};
use globset::{Glob, GlobSetBuilder};
use is_terminal::IsTerminal;
use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};

pub use crate::codebase::ts_resolver::TsConfig;
pub use graph::{DepGraph, EdgeKind, NodeId};

pub use crate::cli::Format;

include!("args_test_globs.rs");
include!("args_relationships.rs");

include!("traversal_entrypoints.rs");
include!("traversal_validation.rs");
include!("traversal_queue_roots.rs");
include!("symbol_resolution.rs");
include!("shared_traversal.rs");
include!("shared_traversal_facts.rs");
include!("shared_traversal_reports.rs");
include!("shared_graph_cache.rs");
include!("shared_traversal_graph.rs");
include!("shared_traversal_collect.rs");
include!("output_args.rs");
include!("run.rs");

#[cfg(test)]
mod shared_traversal_facts_tests;
#[cfg(test)]
mod traversal;

#[cfg(test)]
mod root_dependency_test_helpers;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod traversal_entrypoint_test_helpers;

fn write_output_results(
    format: Format,
    root_strs: &[String],
    result: &TraversalResult,
    out: &mut dyn Write,
) -> Result<()> {
    write_entries(format, root_strs, &result.entries, &result.root, out)
}

pub(crate) struct TraversalResult {
    entries: Vec<graph::NodeEntry>,
    root: PathBuf,
}

pub(crate) fn collect_and_filter_entries(
    args: &TraverseArgs,
    direction: Direction,
    cwd_early: &Path,
    timings: &mut crate::codebase::timing::PhaseTimings,
) -> Result<TraversalResult> {
    let root = resolve_root(args, cwd_early);
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let allowed = relationship_filter(&args.relationships);
    let build_plan = graph::GraphBuildPlan::from_allowed(allowed.as_ref())
        .with_symbols(traversal_needs_symbol_facts(args));
    let mut framework_plan =
        crate::codebase::test_discovery::FrameworkPreparationPlan::for_graph(build_plan);
    framework_plan.include_framework_names(args.tests.iter().map(String::as_str));
    let mut shared = SharedTraversalContext::prepare_with_framework_plan(
        root,
        args.tsconfig.as_deref(),
        None,
        build_plan,
        framework_plan,
    )?;

    timings.mark("search");
    timings.mark("ingest");
    let result = collect_and_filter_entries_shared(args, direction, cwd_early, &mut shared)?;
    timings.mark("parse");
    timings.mark("analysis");
    Ok(result)
}

fn apply_filters(
    entries: Vec<graph::NodeEntry>,
    args: &TraverseArgs,
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    tsconfig: &TsConfig,
    visible_paths: &crate::codebase::ts_source::VisiblePathSnapshot,
    prepared_test_projects: Option<&crate::codebase::test_discovery::PreparedTestProjects>,
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
    prepared_test_projects: Option<&crate::codebase::test_discovery::PreparedTestProjects>,
) -> Vec<String> {
    let runner = crate::codebase::test_discovery::TestRunner::from_name(framework);
    if let Some(runner) = runner {
        let root_visible_paths = visible_paths.paths_for(root);
        let discovered = match prepared_test_projects {
            Some(prepared) => {
                crate::codebase::test_discovery::discover_tests_from_prepared_projects(
                    root,
                    config,
                    runner,
                    prepared,
                    &root_visible_paths,
                    tsconfig,
                )
            }
            None => crate::codebase::test_discovery::discover_tests_from_visible(
                root,
                config,
                runner,
                &root_visible_paths,
                tsconfig,
            ),
        };
        if let Ok(discovered) = discovered {
            let filters = discovered
                .tests
                .iter()
                .map(|path| {
                    crate::codebase::test_discovery::literal_path_glob(
                        &crate::codebase::ts_source::relative_slash_path(root, path),
                    )
                })
                .collect::<Vec<_>>();
            if filters.is_empty() {
                return test_globs(framework);
            }
            return filters;
        }
    }
    test_globs(framework)
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
            graph::NodeId::Module(specifier) => filter.is_match(specifier),
            _ => false,
        })
        .collect())
}
