use super::render::{render, resolve_format, to_json, Report};
use super::reverse::{build_index, importer_paths};
use super::shared::{read_symbols, rel_str, resolve_target};
use crate::cli::Format;
use anyhow::Result;
use is_terminal::IsTerminal;
use serde::Serialize;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

/// `dead-exports`: do any files still import the named exports? With no names,
/// checks every export of the file. Exits non-zero when any export is dead.
#[derive(clap::Parser, Debug)]
pub struct DeadExportsArgs {
    /// The TS/JS file that defines the exports (relative to --root or absolute).
    #[arg(value_name = "FILE")]
    pub file: PathBuf,

    /// Specific export names to check. Defaults to every export of the file.
    #[arg(value_name = "NAME")]
    pub names: Vec<String>,

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
struct DeadResult {
    name: String,
    referenced: bool,
    importer_count: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeadExportsReport {
    file: String,
    results: Vec<DeadResult>,
    any_dead: bool,
}

impl DeadExportsReport {
    fn exit_code(&self) -> ExitCode {
        if self.any_dead {
            ExitCode::FAILURE
        } else {
            ExitCode::SUCCESS
        }
    }
}

fn compute(args: &DeadExportsArgs) -> Result<DeadExportsReport> {
    let target = resolve_target(&args.file, args.root.as_deref(), args.tsconfig.as_deref())?;
    let names = if args.names.is_empty() {
        read_symbols(&target.abs_file)?
            .exports
            .iter()
            .map(|export| export.name.clone())
            .collect()
    } else {
        args.names.clone()
    };
    let index = build_index(&target)?;

    let results: Vec<DeadResult> = names
        .iter()
        .map(|name| {
            let importers = importer_paths(&index, &target.abs_file, name, &target.root);
            DeadResult {
                name: name.clone(),
                referenced: !importers.is_empty(),
                importer_count: importers.len(),
            }
        })
        .collect();

    let any_dead = results.iter().any(|result| !result.referenced);
    Ok(DeadExportsReport {
        file: rel_str(&target.abs_file, &target.root),
        results,
        any_dead,
    })
}

impl Report for DeadExportsReport {
    fn write_human(&self, w: &mut dyn Write) -> io::Result<()> {
        writeln!(w, "{}", self.file)?;
        for result in &self.results {
            if result.referenced {
                writeln!(
                    w,
                    "  {}: referenced ({})",
                    result.name, result.importer_count
                )?;
            } else {
                writeln!(w, "  {}: DEAD", result.name)?;
            }
        }
        Ok(())
    }

    fn write_paths(&self, w: &mut dyn Write) -> io::Result<()> {
        for result in &self.results {
            if !result.referenced {
                writeln!(w, "{}", result.name)?;
            }
        }
        Ok(())
    }
}

pub fn run(args: DeadExportsArgs) -> Result<ExitCode> {
    let report = compute(&args)?;
    let format = resolve_format(args.json, args.format, io::stdout().is_terminal());
    let stdout = io::stdout();
    let mut out = stdout.lock();
    render(&report, format, &mut out)?;
    Ok(report.exit_code())
}

pub fn run_json(args: DeadExportsArgs) -> Result<String> {
    to_json(&compute(&args)?)
}

#[cfg(test)]
mod tests;
