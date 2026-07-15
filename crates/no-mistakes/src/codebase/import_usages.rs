use crate::cli::Format;
use crate::codebase::dependencies::graph::{GraphFiles, TsFactLookup};
use crate::codebase::ts_source::facts::{
    collect_ts_facts_with_session_and_context, TsFactContext, TsFactPlan,
};
use crate::codebase::ts_source::relative_slash_path;
use anyhow::{Context, Result};
use is_terminal::IsTerminal;
use model::{import_usage, ImportUsageFile, ImportUsagesReport};
use rayon::prelude::*;
use std::io;
use std::path::{Path, PathBuf};

mod model;
mod output;
mod paths;

pub(crate) struct PreparedImportUsages {
    graph_files: GraphFiles,
    roots: Vec<String>,
}

impl PreparedImportUsages {
    pub(crate) fn files(&self) -> &[PathBuf] {
        self.graph_files.all()
    }
}

#[derive(clap::Parser, Debug)]
pub struct ImportUsagesArgs {
    /// TS/JS files to inspect. If omitted, scans discovered source files.
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,

    /// Project root directory (default: current working directory).
    #[arg(long, value_name = "PATH")]
    pub root: Option<PathBuf>,

    /// Additional root directories to scan, relative to --root or absolute.
    #[arg(long = "scan-root", value_name = "PATH")]
    pub scan_roots: Vec<PathBuf>,

    /// Only include source files matching this root-relative glob. Repeatable.
    #[arg(long = "filter", value_name = "GLOB")]
    pub filters: Vec<String>,

    /// Output format: json, md, yml, paths, human.
    #[arg(long, value_name = "FORMAT", conflicts_with = "json")]
    pub format: Option<Format>,

    /// Shorthand for `--format json`.
    #[arg(long, default_value_t = false, conflicts_with = "format")]
    pub json: bool,

    /// Legacy programmatic timing switch. CLI timing flags are root-global.
    #[arg(skip)]
    pub timings: bool,
}

pub fn run(args: ImportUsagesArgs) -> Result<()> {
    let _diagnostics = crate::diagnostics::LegacyDiagnosticsGuard::new(args.timings, false);
    let mut timings = crate::codebase::timing::PhaseTimings::start();
    let report = collect_with_timings(&args, Some(&mut timings))?;
    let format = output::resolve_format(args.json, args.format, io::stdout().is_terminal());
    output::write_report(&report, format, &mut io::stdout().lock())?;
    timings.mark("output");
    if args.timings {
        timings.print_stderr();
    }
    Ok(())
}

pub fn run_json(args: ImportUsagesArgs) -> Result<String> {
    let report = collect(&args)?;
    serde_json::to_string_pretty(&report).context("import usages JSON output must be UTF-8")
}

pub fn collect(args: &ImportUsagesArgs) -> Result<ImportUsagesReport> {
    collect_with_timings(args, None)
}

pub(crate) fn prepare_file_universe(
    args: &ImportUsagesArgs,
    root: &Path,
    cwd: &Path,
    session: &crate::codebase::analysis_session::AnalysisSession,
) -> Result<PreparedImportUsages> {
    let files = paths::resolve_files_with_session(session, args, root, cwd)?;
    Ok(PreparedImportUsages {
        graph_files: GraphFiles::from_files(files),
        roots: paths::roots_for_output(args, root),
    })
}

pub(crate) fn collect_with_facts(
    root: &Path,
    prepared: &PreparedImportUsages,
    facts: &dyn TsFactLookup,
) -> Result<ImportUsagesReport> {
    collect_from_facts(root, prepared.roots.clone(), &prepared.graph_files, facts)
}

fn collect_with_timings(
    args: &ImportUsagesArgs,
    mut timings: Option<&mut crate::codebase::timing::PhaseTimings>,
) -> Result<ImportUsagesReport> {
    let cwd = std::env::current_dir().context("reading current directory")?;
    let root = paths::normalize_root(args.root.as_deref(), &cwd);
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current());
    let prepared = prepare_file_universe(args, &root, &cwd, &session)?;
    if let Some(timings) = &mut timings {
        timings.mark("search");
    }

    let facts = collect_ts_facts_with_session_and_context(
        &session,
        prepared.files(),
        TsFactPlan::imports(),
        &TsFactContext::default(),
    );
    if let Some(timings) = &mut timings {
        timings.mark("parse");
    }
    let report = collect_with_facts(&root, &prepared, &facts)?;
    if let Some(timings) = &mut timings {
        timings.mark("analysis");
    }
    Ok(report)
}

pub(crate) fn collect_from_facts(
    root: &Path,
    roots: Vec<String>,
    graph_files: &GraphFiles,
    facts: &dyn TsFactLookup,
) -> Result<ImportUsagesReport> {
    let mut files: Vec<ImportUsageFile> = graph_files
        .indexable()
        .par_iter()
        .filter_map(|path| {
            let file_facts = facts.get_ts_facts(path)?;
            let mut imports: Vec<_> = file_facts.imports.iter().map(import_usage).collect();
            imports.sort_by(|a, b| {
                (a.line, a.kind, &a.specifier).cmp(&(b.line, b.kind, &b.specifier))
            });
            Some(ImportUsageFile {
                path: relative_slash_path(root, path),
                imports,
            })
        })
        .collect();
    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(ImportUsagesReport { roots, files })
}

#[cfg(test)]
mod coverage_tests;
#[cfg(test)]
mod tests;
