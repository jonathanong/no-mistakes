use crate::check_runner;
use anyhow::{Context, Result};
use clap::Args;
use no_mistakes::cli::{resolve_root, Format};
use std::path::PathBuf;
use std::process::ExitCode;
mod output;

#[derive(Args, Debug)]
pub(crate) struct CheckArgs {
    /// Project root directory.
    #[arg(long, default_value = ".", global = true)]
    root: PathBuf,
    /// Path to config file.
    #[arg(long, global = true)]
    config: Option<PathBuf>,
    /// Path to tsconfig.json for queue import alias resolution.
    #[arg(long, global = true)]
    tsconfig: Option<PathBuf>,
    /// Output format: json, yml, md, paths, human.
    #[arg(
        long,
        value_enum,
        default_value = "human",
        global = true,
        conflicts_with = "json"
    )]
    format: Format,
    /// Shorthand for --format json.
    #[arg(long, global = true, conflicts_with = "format")]
    json: bool,
    /// Legacy programmatic timing switch. CLI timing flags are root-global.
    #[arg(skip)]
    timings: bool,
    /// Print fine-grained timing for internal hot paths to stderr (which
    /// `rules` sub-check dominates, which dependency-graph edge kind is
    /// expensive, which Playwright analysis step is slow). Implies
    /// `--timings`-level detail is not enough on its own to diagnose a
    /// regression; use this instead of a special instrumented build.
    #[arg(skip)]
    verbose_timings: bool,
}

pub(crate) fn run(args: CheckArgs) -> Result<ExitCode> {
    let _diagnostics = no_mistakes::diagnostics::LegacyDiagnosticsGuard::new(
        args.timings || args.verbose_timings,
        args.verbose_timings,
    );
    let cwd = std::env::current_dir().context("cwd must be accessible")?;
    let root = resolve_root(&args.root, &cwd);
    let results = check_runner::run_all(root, args.config, args.tsconfig)?;
    record_missing_check_timings(&results);
    no_mistakes::invocation::commit_timeout()?;
    for warning in &results.warnings {
        eprintln!("{warning}");
    }

    let has_failures = has_failures(&results);
    let format = if args.json { Format::Json } else { args.format };
    output::print(&results, format);

    Ok(if has_failures {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

fn record_missing_check_timings(results: &check_runner::CheckResults) {
    let Some(observer) = no_mistakes::diagnostics::current() else {
        return;
    };
    let existing = observer
        .snapshot()
        .timings
        .into_iter()
        .map(|entry| entry.label)
        .collect::<std::collections::HashSet<_>>();
    for (label, duration) in &results.timings {
        let (label, kind) = match *label {
            "discover" => ("discovery", no_mistakes::diagnostics::TimingKind::Serial),
            "parse_extract" => ("parse", no_mistakes::diagnostics::TimingKind::Serial),
            "react" => (
                "analysis.react",
                no_mistakes::diagnostics::TimingKind::Parallel,
            ),
            "queues" => (
                "analysis.queues",
                no_mistakes::diagnostics::TimingKind::Parallel,
            ),
            "rules" => (
                "analysis.rules",
                no_mistakes::diagnostics::TimingKind::Parallel,
            ),
            "integration" => (
                "analysis.integration",
                no_mistakes::diagnostics::TimingKind::Parallel,
            ),
            "codebase" => (
                "analysis.codebase",
                no_mistakes::diagnostics::TimingKind::Parallel,
            ),
            "filesystem_rules" => (
                "analysis.filesystem_rules",
                no_mistakes::diagnostics::TimingKind::Parallel,
            ),
            _ => continue,
        };
        if !existing.contains(label) {
            observer.record_duration(label, *duration, kind);
        }
    }
}

fn has_failures(results: &check_runner::CheckResults) -> bool {
    !results.react.is_empty()
        || !results.queues.is_empty()
        || !results.rules.is_empty()
        || !results.integration.is_empty()
        || !results.codebase.is_empty()
        || !results.warnings.is_empty()
}
