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

include!("args_relationships.rs");

include!("traversal_entrypoints.rs");
include!("traversal.rs");
include!("traversal_queue_roots.rs");
include!("symbol_resolution.rs");
include!("shared_traversal.rs");
include!("output_args.rs");

#[cfg(test)]
mod tests;

pub fn run(args: TraverseArgs, direction: Direction) -> Result<()> {
    let cwd_early = std::env::current_dir().context("reading current directory")?;
    let mut timings = crate::codebase::timing::PhaseTimings::start();

    let result = collect_and_filter_entries(&args, direction, &cwd_early, &mut timings)?;
    let output_result = output_results(&args, &result);

    timings.mark("output");
    if args.timings {
        timings.print_stderr();
    }

    output_result?;

    Ok(())
}

pub fn run_json(args: TraverseArgs, direction: Direction) -> Result<String> {
    let cwd_early = std::env::current_dir().context("reading current directory")?;
    let mut timings = crate::codebase::timing::PhaseTimings::start();
    let result = collect_and_filter_entries(&args, direction, &cwd_early, &mut timings)?;
    let root_strs = output_root_strings(&args);
    let mut out = Vec::new();
    write_output_results(Format::Json, &root_strs, &result, &mut out)?;
    String::from_utf8(out).context("dependency JSON output must be UTF-8")
}

pub(crate) fn result_json(args: &TraverseArgs, result: &TraversalResult) -> Result<String> {
    let root_strs = output_root_strings(args);
    let mut out = Vec::new();
    write_output_results(Format::Json, &root_strs, result, &mut out)?;
    String::from_utf8(out).context("dependency JSON output must be UTF-8")
}

fn output_results(args: &TraverseArgs, result: &TraversalResult) -> Result<()> {
    let root_strs = output_root_strings(args);

    let stdout = io::stdout();
    let stdout_is_terminal = stdout.is_terminal();
    let mut out = stdout.lock();

    let format = resolve_format(args.json, args.format, stdout_is_terminal);
    write_output_results(format, &root_strs, result, &mut out)
}

fn output_root_strings(args: &TraverseArgs) -> Vec<String> {
    args.files
        .iter()
        .enumerate()
        .map(|(index, file)| {
            let file = file.display().to_string();
            match args.file_symbols.get(index).and_then(Option::as_deref) {
                Some(symbol) => format!("{file}#{symbol}"),
                None => file,
            }
        })
        .collect()
}

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

    let tsconfig = resolve_tsconfig(args, &root)?;
    let graph_files = graph::GraphFiles::discover(&root);
    let entrypoints = resolve_entrypoints_with_files(
        &args.files,
        &args.file_symbols,
        &args.file_entrypoints_are_structured,
        &root,
        cwd_early,
        &graph_files,
        args.include_symbols,
    );

    timings.mark("search");

    // Check for #symbol used in Deps direction (unsupported).
    validate_direction(&direction, &entrypoints)?;

    let allowed = relationship_filter(&args.relationships);
    let build_plan =
        graph::GraphBuildPlan::from_allowed(allowed.as_ref()).with_symbols(args.include_symbols);
    let ctx = TraversalCtx {
        root: &root,
        tsconfig: &tsconfig,
        graph_files: &graph_files,
        build_plan,
        allowed: allowed.as_ref(),
        symbols: args.include_symbols,
    };
    let roots: Vec<NodeId> = entrypoints.iter().map(|e| e.node.clone()).collect();
    let import_only = !args.include_symbols && relationships_are_import_only(&args.relationships);

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
        all_filters.extend(test_filters(root, framework));
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

fn test_filters(root: &Path, framework: &str) -> Vec<String> {
    let runner = match framework {
        "vitest" => Some(crate::codebase::test_discovery::TestRunner::Vitest),
        "playwright" => Some(crate::codebase::test_discovery::TestRunner::Playwright),
        "swift" => Some(crate::codebase::test_discovery::TestRunner::Swift),
        _ => None,
    };
    if let Some(runner) = runner {
        if let Ok(config) = crate::config::v2::load_v2_config(root, None) {
            if let Ok(Some(filters)) =
                crate::codebase::test_discovery::discovered_test_globs(root, &config, runner)
            {
                return filters;
            }
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
