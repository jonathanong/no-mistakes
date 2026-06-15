use super::call_sites_visit::collect_call_sites;
use super::render::{render, resolve_format, to_json, Report};
use super::reverse::build_index;
use super::shared::{rel_str, resolve_target};
use crate::cli::Format;
use crate::codebase::dependencies::graph::SymbolIndex;
use anyhow::Result;
use is_terminal::IsTerminal;
use rayon::prelude::*;
use serde::Serialize;
use std::collections::{BTreeSet, HashMap, HashSet};
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
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CallSitesReport {
    file: String,
    export: String,
    call_sites: Vec<CallSite>,
}

/// Map every file that may call the export to the local name(s) it is bound to.
/// The defining file is searched under the export's own name (internal calls).
fn local_names_by_file(
    index: &SymbolIndex,
    abs_file: &Path,
    export_name: &str,
) -> HashMap<PathBuf, HashSet<String>> {
    let mut by_file: HashMap<PathBuf, HashSet<String>> = HashMap::new();
    by_file
        .entry(abs_file.to_path_buf())
        .or_default()
        .insert(export_name.to_string());
    if let Some(records) = index.importers_of(abs_file, export_name) {
        for (importer, local, _is_reexport) in records {
            by_file
                .entry(importer.clone())
                .or_default()
                .insert(local.clone());
        }
    }
    by_file
}

fn sites_for_file(path: &Path, names: &HashSet<String>, root: &Path) -> Vec<CallSite> {
    let Ok(source) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    crate::ast::with_program(path, &source, |program, src| {
        collect_call_sites(program, src, names)
            .into_iter()
            .map(|raw| CallSite {
                file: rel_str(path, root),
                line: raw.line,
                caller: raw.caller,
                arg_count: raw.arg_count,
                has_spread: raw.has_spread,
                args: raw.args,
            })
            .collect()
    })
    .unwrap_or_default()
}

fn compute(args: &CallSitesArgs) -> Result<CallSitesReport> {
    let target = resolve_target(&args.file, args.root.as_deref(), args.tsconfig.as_deref())?;
    let index = build_index(&target)?;
    let by_file = local_names_by_file(&index, &target.abs_file, &args.export_name);

    let mut call_sites: Vec<CallSite> = by_file
        .par_iter()
        .flat_map(|(path, names)| sites_for_file(path, names, &target.root))
        .collect();
    call_sites.sort_by(|a, b| (&a.file, a.line).cmp(&(&b.file, b.line)));

    Ok(CallSitesReport {
        file: rel_str(&target.abs_file, &target.root),
        export: args.export_name.clone(),
        call_sites,
    })
}

impl Report for CallSitesReport {
    fn write_human(&self, w: &mut dyn Write) -> io::Result<()> {
        writeln!(w, "{}#{}", self.file, self.export)?;
        for site in &self.call_sites {
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

#[cfg(test)]
mod tests;
