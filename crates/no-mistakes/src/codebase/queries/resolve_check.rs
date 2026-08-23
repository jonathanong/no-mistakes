use super::render::{render, resolve_format, to_json, Report};
use crate::cli::Format;
use crate::codebase::dependencies::extract::ImportKind;
use anyhow::Result;
use is_terminal::IsTerminal;
use serde::Serialize;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

mod batch;
pub use batch::BatchResolveCheckReport;
use batch::{batch_report, compute_many};

/// `resolve-check`: do all imports in one or more files resolve?
#[derive(clap::Parser, Debug)]
pub struct ResolveCheckArgs {
    /// TS/JS files to check (relative to --root or absolute).
    #[arg(value_name = "FILE", required = true, num_args = 1..)]
    pub files: Vec<PathBuf>,
    /// Project root (default: current working directory).
    #[arg(long, value_name = "PATH")]
    pub root: Option<PathBuf>,
    /// Path to tsconfig.json for alias resolution. If omitted, searches upward.
    #[arg(long, value_name = "FILE")]
    pub tsconfig: Option<PathBuf>,
    /// Output format: json, yml, md, paths, human.
    #[arg(long, value_name = "FORMAT")]
    pub format: Option<Format>,
    /// Shorthand for `--format json`.
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum Status {
    Resolved,
    Unresolved,
    External,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ImportRow {
    specifier: String,
    kind: &'static str,
    status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    resolved: Option<String>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResolveCheckReport {
    file: String,
    all_resolve: bool,
    imports: Vec<ImportRow>,
    /// Specifiers that should have resolved but did not.
    unresolved: Vec<String>,
}

impl ResolveCheckReport {
    fn exit_code(&self) -> ExitCode {
        if self.all_resolve {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        }
    }
}

fn kind_str(kind: ImportKind) -> &'static str {
    match kind {
        ImportKind::Static => "static",
        ImportKind::Type => "type",
        ImportKind::Dynamic => "dynamic",
        ImportKind::Require => "require",
        ImportKind::RequireResolve => "require-resolve",
    }
}

/// Declaration files only satisfy type-only references because they do not
/// emit a runtime module.
fn is_declaration_file(path: &Path) -> bool {
    let name = path.to_string_lossy();
    name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
}

fn compute(args: &ResolveCheckArgs) -> Result<ResolveCheckReport> {
    let mut reports = compute_many(args)?;
    anyhow::ensure!(
        reports.len() == 1,
        "single-file report requested for multiple files"
    );
    Ok(reports.remove(0))
}

impl Report for ResolveCheckReport {
    fn write_human(&self, w: &mut dyn Write) -> io::Result<()> {
        writeln!(w, "{}", self.file)?;
        for row in &self.imports {
            match (row.status, &row.resolved) {
                (Status::Resolved, Some(target)) => {
                    writeln!(w, "  ok       {} -> {}", row.specifier, target)?;
                }
                (Status::Unresolved, _) => writeln!(w, "  MISSING  {}", row.specifier)?,
                _ => writeln!(w, "  external {}", row.specifier)?,
            }
        }
        Ok(())
    }

    fn write_paths(&self, w: &mut dyn Write) -> io::Result<()> {
        for row in &self.imports {
            if let Some(target) = &row.resolved {
                writeln!(w, "{target}")?;
            }
        }
        Ok(())
    }
}

pub fn run(args: ResolveCheckArgs) -> Result<ExitCode> {
    let format = resolve_format(args.json, args.format, io::stdout().is_terminal());
    let stdout = io::stdout();
    let mut out = stdout.lock();
    if args.files.len() == 1 {
        let report = compute(&args)?;
        render(&report, format, &mut out)?;
        Ok(report.exit_code())
    } else {
        let report = batch_report(compute_many(&args)?);
        render(&report, format, &mut out)?;
        Ok(report.exit_code())
    }
}

pub fn run_json(args: ResolveCheckArgs) -> Result<String> {
    if args.files.len() == 1 {
        to_json(&compute(&args)?)
    } else {
        to_json(&batch_report(compute_many(&args)?))
    }
}

/// N-API's `files` option always requests the batch schema, including for a
/// one-element list. The CLI has no such wrapper distinction: one positional
/// file retains its original response shape.
pub fn run_json_batch(args: ResolveCheckArgs) -> Result<String> {
    to_json(&batch_report(compute_many(&args)?))
}

#[cfg(test)]
mod tests;
