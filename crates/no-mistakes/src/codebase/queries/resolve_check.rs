mod resolution;

use super::render::{render, resolve_format, to_json, Report};
use super::shared::resolve_target;
use crate::cli::Format;
use crate::codebase::dependencies::extract::{
    is_tsx_file, ExtractedImport, ImportExtractor, ImportKind,
};
use crate::codebase::ts_resolver::ImportResolver;
use anyhow::{Context, Result};
use is_terminal::IsTerminal;
use resolution::{is_declaration_file, resolve_declaration, resolve_ts_source};
use serde::Serialize;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// `resolve-check`: do all imports in a single file resolve?
#[derive(clap::Parser, Debug)]
pub struct ResolveCheckArgs {
    /// The TS/JS file to check (relative to --root or absolute).
    #[arg(value_name = "FILE")]
    pub file: PathBuf,

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
    /// Resolved to a local file on disk.
    Resolved,
    /// A relative or aliased import whose target is missing — a real error.
    Unresolved,
    /// A bare npm package, Node builtin, or subpath import (not an error).
    External,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ImportRow {
    specifier: String,
    kind: &'static str,
    status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    resolved: Option<String>,
}

#[derive(Serialize)]
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
    }
}

/// Classify one import specifier against a shared resolver and tsconfig aliases.
fn classify(
    imp: &ExtractedImport,
    abs_file: &Path,
    target: &super::shared::Target,
    resolver: &ImportResolver,
) -> ImportRow {
    // Fall back to declaration-file candidates so a *type-only* import of a
    // declaration-only module (`import type { Foo } from './types'` →
    // `types.d.ts` or `types/index.d.ts`) resolves. A value import still needs
    // an emitted module, so the fallback is gated on the import being type-only.
    let resolved = resolver
        .resolve(&imp.specifier, abs_file)
        .or_else(|| resolve_ts_source(&imp.specifier, abs_file, resolver))
        .or_else(|| resolve_declaration(imp, abs_file, resolver))
        // A declaration file has no runtime module, so a value import of one is
        // not actually resolved.
        .filter(|path| imp.kind == ImportKind::Type || !is_declaration_file(path));
    let status = if resolved.is_some() {
        Status::Resolved
    } else if imp.specifier.starts_with('.')
        || ImportResolver::new(&target.tsconfig).matches_alias(&imp.specifier)
    {
        Status::Unresolved
    } else {
        Status::External
    };
    ImportRow {
        specifier: imp.specifier.clone(),
        kind: kind_str(imp.kind),
        status,
        resolved: resolved.map(|abs| super::shared::rel_str(&abs, &target.root)),
    }
}

fn compute(args: &ResolveCheckArgs) -> Result<ResolveCheckReport> {
    let target = resolve_target(&args.file, args.root.as_deref(), args.tsconfig.as_deref())?;
    let source = std::fs::read_to_string(&target.abs_file)
        .context(format!("reading {}", target.abs_file.display()))?;
    let extractor = if is_tsx_file(&target.abs_file) {
        ImportExtractor::for_tsx()?
    } else {
        ImportExtractor::for_typescript()?
    };
    let resolver = ImportResolver::new(&target.tsconfig).without_cache();
    let imports: Vec<ImportRow> = extractor
        .extract(&source)?
        .iter()
        .map(|imp| classify(imp, &target.abs_file, &target, &resolver))
        .collect();

    let unresolved: Vec<String> = imports
        .iter()
        .filter(|row| row.status == Status::Unresolved)
        .map(|row| row.specifier.clone())
        .collect();

    Ok(ResolveCheckReport {
        file: super::shared::rel_str(&target.abs_file, &target.root),
        all_resolve: unresolved.is_empty(),
        imports,
        unresolved,
    })
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
    let report = compute(&args)?;
    let format = resolve_format(args.json, args.format, io::stdout().is_terminal());
    let stdout = io::stdout();
    let mut out = stdout.lock();
    render(&report, format, &mut out)?;
    Ok(report.exit_code())
}

pub fn run_json(args: ResolveCheckArgs) -> Result<String> {
    to_json(&compute(&args)?)
}

#[cfg(test)]
mod tests;
