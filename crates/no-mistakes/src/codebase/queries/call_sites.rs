use super::diagnostics::QueryDiagnostics;
use super::render::{render, resolve_format, to_json, Report};
use super::reverse::{find_export, symbols_from_prepared};
use super::shared::{rel_str, resolve_target};
use crate::cli::Format;
use anyhow::Result;
use is_terminal::IsTerminal;
use serde::Serialize;
use std::collections::BTreeSet;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// `call-sites`: every call site of an exported function, with argument shapes.
#[derive(clap::Parser, Debug)]
pub struct CallSitesArgs {
    /// The TS/JS file that defines the export (relative to --root or absolute).
    #[arg(value_name = "FILE")]
    pub file: PathBuf,

    /// The exported function name to find call sites for.
    #[arg(value_name = "EXPORT")]
    pub export_name: String,

    /// Project root (default: current working directory).
    #[arg(long, value_name = "PATH")]
    pub root: Option<PathBuf>,

    /// Path to tsconfig.json for resolving import specifiers.
    #[arg(long, value_name = "FILE")]
    pub tsconfig: Option<PathBuf>,

    /// Output format: json, yml, md, paths, human.
    #[arg(long, value_name = "FORMAT")]
    pub format: Option<Format>,

    /// Shorthand for `--format json`.
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CallSite {
    file: String,
    line: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    caller: Option<String>,
    arg_count: usize,
    has_spread: bool,
    args: Vec<&'static str>,
    /// Proven name reference. Argument fields are empty and are not a call shape.
    #[serde(skip_serializing_if = "is_false")]
    reference_only: bool,
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CallSitesReport {
    file: String,
    export: String,
    call_sites: Vec<CallSite>,
    #[serde(flatten)]
    diagnostics: std::sync::Arc<QueryDiagnostics>,
}

mod language;
pub(crate) mod prepared;
mod sites;

pub(crate) use sites::PreparedCallSitesProjection;

fn compute(args: &CallSitesArgs) -> Result<CallSitesReport> {
    if let Some(report) = language::from_args(args)? {
        return Ok(report);
    }
    let target = resolve_target(&args.file, args.root.as_deref(), args.tsconfig.as_deref())?;
    let analysis = prepared::prepare(&target)?;
    let symbols = symbols_from_prepared(&target, &analysis.facts)?;
    project_report(
        &target.root,
        &target.abs_file,
        &args.export_name,
        &symbols,
        PreparedCallSitesProjection {
            graph: &analysis.graph,
            facts: &analysis.facts,
            files: analysis.files.indexable(),
            diagnostics: &analysis.diagnostics,
        },
    )
}

pub(crate) fn project_report(
    root: &Path,
    file: &Path,
    export_name: &str,
    symbols: &crate::codebase::ts_symbols::FileSymbols,
    projection: PreparedCallSitesProjection<'_>,
) -> Result<CallSitesReport> {
    let export = find_export(symbols, export_name).filter(|export| !export.is_type_only);
    anyhow::ensure!(
        export.is_some(),
        "`{}` is not a value export of {}",
        export_name,
        rel_str(file, root)
    );
    // The public default declaration name remains accepted alongside `default`.
    let canonical_export = export.map_or(export_name, |export| {
        if export.kind == crate::codebase::ts_symbols::ExportKind::Default {
            "default"
        } else {
            export_name
        }
    });
    Ok(CallSitesReport {
        file: rel_str(file, root),
        export: export_name.to_string(),
        call_sites: sites::project_sites(root, file, canonical_export, &projection),
        diagnostics: projection.diagnostics.clone(),
    })
}

impl Report for CallSitesReport {
    fn write_human(&self, w: &mut dyn Write) -> io::Result<()> {
        writeln!(w, "{}#{}", self.file, self.export)?;
        self.diagnostics.write_human(w)?;
        for site in &self.call_sites {
            if site.reference_only {
                writeln!(w, "  {}:{} {} reference", site.file, site.line, self.export)?;
                continue;
            }
            let caller = site.caller.as_deref().unwrap_or("(top-level)");
            let args = site.args.join(", ");
            writeln!(w, "  {}:{} {caller}({args})", site.file, site.line)?;
        }
        Ok(())
    }

    fn write_paths(&self, w: &mut dyn Write) -> io::Result<()> {
        let unique: BTreeSet<&String> = self.call_sites.iter().map(|site| &site.file).collect();
        for path in unique {
            writeln!(w, "{path}")?;
        }
        Ok(())
    }
}

pub fn run(args: CallSitesArgs) -> Result<ExitCode> {
    let report = compute(&args)?;
    let format = resolve_format(args.json, args.format, io::stdout().is_terminal());
    let stdout = io::stdout();
    let mut out = stdout.lock();
    render(&report, format, &mut out)?;
    Ok(ExitCode::SUCCESS)
}

pub fn run_json(args: CallSitesArgs) -> Result<String> {
    to_json(&compute(&args)?)
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
