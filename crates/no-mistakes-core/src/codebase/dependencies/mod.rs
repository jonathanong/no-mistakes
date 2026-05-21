pub mod extract;
pub mod graph;
pub mod output;

use anyhow::{bail, Context, Result};
use is_terminal::IsTerminal;
use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};

pub use crate::codebase::ts_resolver::TsConfig;
pub use graph::{DepGraph, EdgeKind, NodeId};

pub use crate::cli::Format;

include!("args_relationships.rs");

include!("traversal.rs");
include!("symbol_resolution.rs");
include!("output_args.rs");

#[cfg(test)]
mod tests;

pub fn run(args: TraverseArgs, direction: Direction) -> Result<()> {
    let cwd_early = std::env::current_dir().context("reading current directory")?;
    let mut timings = crate::codebase::timing::PhaseTimings::start();

    let result = collect_and_filter_entries(&args, direction, &cwd_early, &mut timings)?;
    let root_strs: Vec<String> = args.files.iter().map(|f| f.display().to_string()).collect();

    let stdout = io::stdout();
    let stdout_is_terminal = stdout.is_terminal();
    let mut out = stdout.lock();

    let format = resolve_format(args.json, args.format, stdout_is_terminal);
    write_entries(format, &root_strs, &result.entries, &result.root, &mut out)?;

    timings.mark("output");
    if args.timings {
        timings.print_stderr();
    }

    Ok(())
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

    let tsconfig = resolve_tsconfig(args, &root)?;
    let entrypoints = resolve_entrypoints(&args.files, &root, cwd_early);

    timings.mark("search");

    // Check for #symbol used in Deps direction (unsupported).
    validate_direction(&direction, &entrypoints)?;

    let allowed = relationship_filter(&args.relationships);
    let build_plan = graph::GraphBuildPlan::from_allowed(allowed.as_ref());
    let graph_files = graph::GraphFiles::discover(&root);
    let ctx = TraversalCtx {
        root: &root,
        tsconfig: &tsconfig,
        graph_files: &graph_files,
        build_plan,
        allowed: allowed.as_ref(),
    };
    let roots: Vec<NodeId> = entrypoints
        .iter()
        .map(|e| NodeId::File(e.file.clone()))
        .collect();
    let import_only = relationships_are_import_only(&args.relationships);

    timings.mark("ingest");

    let entries = get_entries(
        direction,
        &roots,
        &entrypoints,
        args.depth,
        import_only,
        &ctx,
    );

    timings.mark("parse");

    let entries = apply_filters(entries, args, &root)?;

    timings.mark("analysis");

    Ok(TraversalResult { entries, root })
}

fn apply_filters(
    entries: Vec<graph::NodeEntry>,
    args: &TraverseArgs,
    root: &Path,
) -> Result<Vec<graph::NodeEntry>> {
    // Build combined filter from --filter and --test globs.
    let mut all_filters = args.filters.clone();
    for framework in &args.tests {
        all_filters.extend(test_globs(framework));
    }
    let filter = graph::build_filter(&all_filters)?;
    Ok(graph::apply_filter(entries, filter.as_ref(), root))
}
