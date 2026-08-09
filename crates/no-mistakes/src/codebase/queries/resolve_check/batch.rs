use super::{
    is_declaration_file, kind_str, ImportRow, ResolveCheckArgs, ResolveCheckReport, Status,
};
use crate::codebase::dependencies::extract::{ExtractedImport, ImportKind};
use crate::codebase::queries::render::Report;
use crate::codebase::ts_resolver::ImportResolver;
use anyhow::{Context, Result};
use rayon::prelude::*;
use serde::Serialize;
use std::collections::BTreeSet;
use std::io::{self, Write};
use std::process::ExitCode;

/// The additive batch response. A one-file request retains the historical
/// `ResolveCheckReport` shape so existing CLI and Node consumers do not need
/// to special-case their single-file calls.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchResolveCheckReport {
    all_resolve: bool,
    unresolved_files: Vec<String>,
    results: Vec<ResolveCheckReport>,
}

fn classify(
    imp: &ExtractedImport,
    target: &super::super::shared::Target,
    resolver: &ImportResolver,
) -> ImportRow {
    let resolved = resolver
        .resolve(&imp.specifier, &target.abs_file)
        .filter(|path| imp.kind == ImportKind::Type || !is_declaration_file(path));
    let status = if resolved.is_some() {
        Status::Resolved
    } else if imp.specifier.starts_with('.') || resolver.matches_alias(&imp.specifier) {
        Status::Unresolved
    } else {
        Status::External
    };
    ImportRow {
        specifier: imp.specifier.clone(),
        kind: kind_str(imp.kind),
        status,
        resolved: resolved.map(|abs| super::super::shared::rel_str(&abs, &target.root)),
    }
}

fn compute_target(
    target: &super::super::shared::Target,
    imports: &[ExtractedImport],
) -> Result<ResolveCheckReport> {
    let resolver = ImportResolver::new_in_session(
        target.tsconfig()?,
        Some(target.visible_files()),
        &target.session,
    );
    let imports: Vec<ImportRow> = imports
        .iter()
        .map(|imp| classify(imp, target, &resolver))
        .collect();
    let unresolved: Vec<String> = imports
        .iter()
        .filter(|row| row.status == Status::Unresolved)
        .map(|row| row.specifier.clone())
        .collect();
    Ok(ResolveCheckReport {
        file: super::super::shared::rel_str(&target.abs_file, &target.root),
        all_resolve: unresolved.is_empty(),
        imports,
        unresolved,
    })
}

/// Collect the union import demand once, then classify independent inputs in
/// parallel against their nearest (or explicit) tsconfig. The source facts are
/// recovered parser facts, retaining imports from syntactically malformed
/// files exactly like the project-wide analysis pipeline.
pub(super) fn compute_many(args: &ResolveCheckArgs) -> Result<Vec<ResolveCheckReport>> {
    let targets = super::super::shared::resolve_targets(
        &args.files,
        args.root.as_deref(),
        args.tsconfig.as_deref(),
    )?;
    if let Some(target) = targets.first() {
        target.validate_explicit_tsconfig()?;
    }
    let facts = super::super::reverse::collect_target_import_facts(&targets[0], &targets);
    let mut reports = targets
        .par_iter()
        .map(|target| {
            let facts = facts.get(&target.abs_file).with_context(|| {
                format!("collecting imports from {}", target.abs_file.display())
            })?;
            compute_target(target, &facts.imports)
        })
        .collect::<Result<Vec<_>>>()?;
    reports.sort_by(|left, right| left.file.cmp(&right.file));
    Ok(reports)
}

pub(super) fn batch_report(results: Vec<ResolveCheckReport>) -> BatchResolveCheckReport {
    let unresolved_files: Vec<String> = results
        .iter()
        .filter(|result| !result.all_resolve)
        .map(|result| result.file.clone())
        .collect();
    BatchResolveCheckReport {
        all_resolve: unresolved_files.is_empty(),
        unresolved_files,
        results,
    }
}

impl BatchResolveCheckReport {
    pub(super) fn exit_code(&self) -> ExitCode {
        if self.all_resolve {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        }
    }
}

impl Report for BatchResolveCheckReport {
    fn write_human(&self, w: &mut dyn Write) -> io::Result<()> {
        for (index, result) in self.results.iter().enumerate() {
            if index > 0 {
                writeln!(w)?;
            }
            result.write_human(w)?;
        }
        Ok(())
    }

    fn write_md(&self, w: &mut dyn Write) -> io::Result<()> {
        for (index, result) in self.results.iter().enumerate() {
            if index > 0 {
                writeln!(w)?;
            }
            writeln!(w, "## {}", result.file)?;
            for row in &result.imports {
                match (row.status, &row.resolved) {
                    (Status::Resolved, Some(target)) => {
                        writeln!(w, "- ok: `{}` → `{target}`", row.specifier)?;
                    }
                    (Status::Unresolved, _) => writeln!(w, "- **MISSING:** `{}`", row.specifier)?,
                    _ => writeln!(w, "- external: `{}`", row.specifier)?,
                }
            }
        }
        Ok(())
    }

    fn write_paths(&self, w: &mut dyn Write) -> io::Result<()> {
        let paths: BTreeSet<&str> = self
            .results
            .iter()
            .flat_map(|result| {
                result
                    .imports
                    .iter()
                    .filter_map(|row| row.resolved.as_deref())
            })
            .collect();
        for path in paths {
            writeln!(w, "{path}")?;
        }
        Ok(())
    }
}
