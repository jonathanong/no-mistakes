use super::load;
use super::{ensure_list_mode, requests_language_file};
use crate::cli::Format;
use crate::codebase::symbols::{Include, SymbolsArgs};
use anyhow::{bail, Result};
use is_terminal::IsTerminal;
use serde::Serialize;
use std::io::{self, Write};

mod text;

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod coverage_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod text_tests;

pub(crate) fn run_symbols(args: SymbolsArgs) -> Result<()> {
    let _diagnostics = crate::diagnostics::LegacyDiagnosticsGuard::new(args.timings, false);
    let mut timings = crate::codebase::timing::PhaseTimings::start();
    let format = resolve_format(args.json, args.format, io::stdout().is_terminal());
    let stdout = io::stdout();
    let mut out = stdout.lock();
    write_symbols(&args, format, &mut out)?;
    timings.mark("output");
    if args.timings {
        timings.print_stderr();
    }
    Ok(())
}

pub(super) fn write_symbols(args: &SymbolsArgs, format: Format, w: &mut dyn Write) -> Result<()> {
    if !requests_language_file(&args.files) {
        bail!("symbols language writer requires a language file");
    }
    ensure_list_mode(args)?;
    let report = collect(args)?;
    match format {
        Format::Json => {
            serde_json::to_writer(&mut *w, &report)?;
            writeln!(w)?;
        }
        Format::Yml => serde_yaml::to_writer(w, &report)?,
        Format::Paths => text::write_paths(&report, w)?,
        Format::Md => text::write_md(&report, w)?,
        Format::Human => text::write_human(&report, w)?,
    }
    Ok(())
}

fn collect(args: &SymbolsArgs) -> Result<Output> {
    let want_exports = matches!(args.include, Include::Exports | Include::Both);
    let want_imports = matches!(args.include, Include::Imports | Include::Both);
    let mut files = Vec::new();
    for file in &args.files {
        let Some(view) = load::prepare_file(file, args.root.as_deref(), args.config.as_deref())?
        else {
            bail!("pass only language files, or only TypeScript files, to symbols");
        };
        files.push(file_output(
            &view,
            want_exports,
            want_imports,
            args.kinds.is_empty(),
        ));
    }
    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(Output {
        roots: args
            .files
            .iter()
            .map(|file| file.display().to_string())
            .collect(),
        files,
    })
}

fn file_output(
    view: &super::view::View,
    want_exports: bool,
    want_imports: bool,
    all_kinds: bool,
) -> FileOutput {
    let mut exports = Vec::new();
    if want_exports && all_kinds {
        let mut names = view.declarations.clone();
        names.sort();
        names.dedup();
        for name in names {
            let Some(line) = view.declaration_line(&name) else {
                continue;
            };
            exports.push(Named {
                name,
                kind: "declaration",
                line,
            });
        }
    }
    let mut imports = Vec::new();
    if want_imports {
        let mut specifiers = view.imports.clone();
        specifiers.sort();
        specifiers.dedup();
        for specifier in specifiers {
            let Some(line) = view.import_line(&specifier) else {
                continue;
            };
            imports.push(ImportRow {
                source: specifier.clone(),
                imported: specifier.clone(),
                local: specifier,
                line,
                type_only: false,
            });
        }
    }
    FileOutput {
        path: view.rel_file.clone(),
        exports,
        imports,
    }
}

fn resolve_format(json: bool, format: Option<Format>, stdout_is_terminal: bool) -> Format {
    if json {
        Format::Json
    } else if let Some(format) = format {
        format
    } else if stdout_is_terminal {
        Format::Human
    } else {
        Format::Json
    }
}

#[derive(Serialize)]
struct Output {
    roots: Vec<String>,
    files: Vec<FileOutput>,
}

#[derive(Serialize)]
struct FileOutput {
    path: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    exports: Vec<Named>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    imports: Vec<ImportRow>,
}

#[derive(Serialize)]
struct Named {
    name: String,
    kind: &'static str,
    line: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ImportRow {
    source: String,
    imported: String,
    local: String,
    line: u32,
    type_only: bool,
}
