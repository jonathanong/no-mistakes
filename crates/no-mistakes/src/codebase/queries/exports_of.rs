use super::render::{render, resolve_format, to_json, Report};
use super::reverse::{build_reverse_analysis, export_importer_paths, export_kind_str};
use super::shared::{read_symbols, rel_str, resolve_target};
use crate::cli::Format;
use crate::codebase::ts_resolver::ImportResolver;
use crate::codebase::ts_symbols::ExportKind;
use anyhow::Result;
use is_terminal::IsTerminal;
use serde::Serialize;
use std::collections::BTreeSet;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

/// `exports-of`: list a file's named exports and who imports each.
#[derive(clap::Parser, Debug)]
pub struct ExportsOfArgs {
    /// The TS/JS file to inspect (relative to --root or absolute).
    #[arg(value_name = "FILE")]
    pub file: PathBuf,

    /// Project root (default: current working directory).
    #[arg(long, value_name = "PATH")]
    pub root: Option<PathBuf>,

    /// Path to tsconfig.json for resolving import specifiers.
    #[arg(long, value_name = "FILE")]
    pub tsconfig: Option<PathBuf>,

    /// Skip the reverse import scan and only list exports (instant, no importers).
    #[arg(long, default_value_t = false)]
    pub no_importers: bool,

    /// Output format: json, yml, md, paths, human.
    #[arg(long, value_name = "FORMAT")]
    pub format: Option<Format>,

    /// Shorthand for `--format json`.
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Serialize)]
struct ExportRow {
    name: String,
    kind: &'static str,
    line: u32,
    /// Resolved re-export target, root-relative. Only set for re-exports.
    #[serde(skip_serializing_if = "Option::is_none")]
    resolved: Option<String>,
    importers: Vec<String>,
}

#[derive(Serialize)]
pub struct ExportsOfReport {
    file: String,
    exports: Vec<ExportRow>,
}

fn compute(args: &ExportsOfArgs) -> Result<ExportsOfReport> {
    let target = resolve_target(&args.file, args.root.as_deref(), args.tsconfig.as_deref())?;
    let (symbols, analysis) = if args.no_importers {
        (read_symbols(&target.abs_file, &target.sources)?, None)
    } else {
        let analysis = build_reverse_analysis(&target)?;
        (analysis.symbols(&target)?, Some(analysis))
    };
    // Target re-export rendering retains the single-file command's nearest
    // visible-tsconfig semantics. Keep it here so reverse-only commands do
    // not eagerly resolve a target config they never use.
    let resolver_config = target.tsconfig()?;
    let resolver = ImportResolver::new(resolver_config).with_visible(&target.visible_files);

    let exports = symbols
        .exports
        .iter()
        .map(|export| {
            let resolved = if let ExportKind::ReExport { source, .. } = &export.kind {
                resolver
                    .resolve(source, &target.abs_file)
                    .map(|abs| rel_str(&abs, &target.root))
            } else {
                None
            };
            let importers = analysis
                .as_ref()
                .map(|analysis| {
                    export_importer_paths(&analysis.index, &target.abs_file, export, &target.root)
                })
                .unwrap_or_default();
            ExportRow {
                name: export.name.clone(),
                kind: export_kind_str(&export.kind),
                line: export.line,
                resolved,
                importers,
            }
        })
        .collect();

    Ok(ExportsOfReport {
        file: rel_str(&target.abs_file, &target.root),
        exports,
    })
}

impl Report for ExportsOfReport {
    fn write_human(&self, w: &mut dyn Write) -> io::Result<()> {
        writeln!(w, "{}", self.file)?;
        for export in &self.exports {
            let consumers = if export.importers.is_empty() {
                "(no importers)".to_string()
            } else {
                export.importers.join(", ")
            };
            let line = format!(
                "  {} ({}) line {} <- {}",
                export.name, export.kind, export.line, consumers
            );
            writeln!(w, "{line}")?;
        }
        Ok(())
    }

    fn write_paths(&self, w: &mut dyn Write) -> io::Result<()> {
        let unique: BTreeSet<&String> = self
            .exports
            .iter()
            .flat_map(|export| export.importers.iter())
            .collect();
        for path in unique {
            writeln!(w, "{path}")?;
        }
        Ok(())
    }
}

pub fn run(args: ExportsOfArgs) -> Result<ExitCode> {
    let report = compute(&args)?;
    let format = resolve_format(args.json, args.format, io::stdout().is_terminal());
    let stdout = io::stdout();
    let mut out = stdout.lock();
    render(&report, format, &mut out)?;
    Ok(ExitCode::SUCCESS)
}

pub fn run_json(args: ExportsOfArgs) -> Result<String> {
    to_json(&compute(&args)?)
}

#[cfg(test)]
mod tests;
