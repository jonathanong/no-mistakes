mod kind;
mod lines;
mod load;
mod native;
mod symbols;
mod view;

use crate::cli::Format;
use crate::codebase::symbols::{SymbolsArgs, SymbolsMode};
use anyhow::{bail, Result};
use std::path::Path;

pub(crate) use symbols::run_symbols;

pub(crate) struct DeadExportRow {
    pub name: String,
    pub referenced: bool,
    pub importer_count: usize,
}

pub(crate) struct DeadExportRows {
    pub file: String,
    pub rows: Vec<DeadExportRow>,
    pub diagnostics: crate::codebase::queries::diagnostics::QueryDiagnostics,
}

pub(crate) struct CallSiteRow {
    pub file: String,
    pub line: u32,
}

pub(crate) struct CallSiteRows {
    pub file: String,
    pub sites: Vec<CallSiteRow>,
    pub diagnostics: crate::codebase::queries::diagnostics::QueryDiagnostics,
}

pub(crate) fn requests_language_file(files: &[impl AsRef<Path>]) -> bool {
    files
        .iter()
        .any(|file| load::frontend(file.as_ref()).is_some())
}

pub(crate) fn language_dead_rows(
    file: &Path,
    names: &[String],
    root: Option<&Path>,
    config: Option<&Path>,
) -> Result<Option<DeadExportRows>> {
    let Some(view) = load::prepare_file(file, root, config)? else {
        return Ok(None);
    };
    let mut wanted: Vec<String> = if names.is_empty() {
        view.declarations.clone()
    } else {
        names.to_vec()
    };
    wanted.sort();
    wanted.dedup();
    let rows = wanted
        .into_iter()
        .map(|name| {
            let importer_count = view.reference_files(&name).len();
            DeadExportRow {
                referenced: importer_count > 0,
                importer_count,
                name,
            }
        })
        .collect();
    let diagnostics = crate::codebase::queries::diagnostics::QueryDiagnostics::operational_gaps(
        &view.root,
        &view.unread_files(),
    );
    Ok(Some(DeadExportRows {
        file: view.rel_file,
        rows,
        diagnostics,
    }))
}

pub(crate) fn language_call_rows(
    file: &Path,
    export_name: &str,
    root: Option<&Path>,
    config: Option<&Path>,
) -> Result<Option<CallSiteRows>> {
    let Some(view) = load::prepare_file(file, root, config)? else {
        return Ok(None);
    };
    if !view.declarations.iter().any(|name| name == export_name) {
        bail!("`{export_name}` is not a declaration of {}", view.rel_file);
    }
    let mut sites: Vec<CallSiteRow> = view
        .reference_files(export_name)
        .into_iter()
        .filter_map(|path| {
            let line = lines::first_name_line(&view.sources, &path, export_name)?;
            Some(CallSiteRow {
                file: crate::codebase::queries::shared::rel_str(&path, &view.root),
                line,
            })
        })
        .collect();
    sites.sort_by(|left, right| (&left.file, left.line).cmp(&(&right.file, right.line)));
    let diagnostics = crate::codebase::queries::diagnostics::QueryDiagnostics::operational_gaps(
        &view.root,
        &view.unread_files(),
    );
    Ok(Some(CallSiteRows {
        file: view.rel_file,
        sites,
        diagnostics,
    }))
}

pub(crate) fn symbols_json(args: &SymbolsArgs) -> Result<Option<String>> {
    if !requests_language_file(&args.files) {
        return Ok(None);
    }
    let mut output = Vec::new();
    symbols::write_symbols(args, Format::Json, &mut output)?;
    Ok(Some(
        String::from_utf8(output).expect("language symbols JSON is UTF-8"),
    ))
}

pub(crate) fn ensure_list_mode(args: &SymbolsArgs) -> Result<()> {
    if args.mode == SymbolsMode::SignatureImpact {
        bail!(
            "signature-impact reads TypeScript exports; language files have name references only"
        );
    }
    if args.files.iter().any(|file| load::frontend(file).is_none()) {
        bail!("pass only language files, or only TypeScript files, to symbols");
    }
    Ok(())
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod branch_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod lines_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod native_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod reference_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod unread_tests;
