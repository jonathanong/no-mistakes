//! Pure helpers for the `react usages` pipeline: include parsing, target
//! splitting, symbol matching, story/test classification, and prop type lookup.

use crate::codebase::ts_symbols::{extract_symbols, ExportKind};
use anyhow::Result;
use std::collections::BTreeSet;
use std::path::Path;

/// Which optional sections of the usages report to populate. `callsites` are
/// always included; the rest are gated so `--include` can distinguish "not
/// requested" (`None`) from "requested but empty" (`Some([])`).
pub struct UsagesInclude {
    pub stories: bool,
    pub tests: bool,
    pub prop_types: bool,
}

impl UsagesInclude {
    pub fn all() -> Self {
        Self {
            stories: true,
            tests: true,
            prop_types: true,
        }
    }

    /// Parse a comma-separated `--include` spec (`stories,tests,props`).
    /// `None` means include everything.
    pub fn parse(spec: Option<&str>) -> Result<Self> {
        let Some(spec) = spec else {
            return Ok(Self::all());
        };
        let mut include = Self {
            stories: false,
            tests: false,
            prop_types: false,
        };
        for part in spec.split(',') {
            match part.trim() {
                "" => {}
                "stories" => include.stories = true,
                "tests" => include.tests = true,
                "props" => include.prop_types = true,
                other => anyhow::bail!(
                    "unknown --include section: {other} (expected stories, tests, props)"
                ),
            }
        }
        Ok(include)
    }
}

/// Split `path#Symbol` into the path and optional symbol (empty symbol → `None`).
pub(crate) fn split_target(target: &str) -> (&str, Option<String>) {
    match target.split_once('#') {
        Some((path, sym)) if !sym.is_empty() => (path, Some(sym.to_string())),
        Some((path, _)) => (path, None),
        None => (target, None),
    }
}

pub(crate) fn callsite_symbol_matches(exported: &str, symbol: Option<&str>) -> bool {
    symbol.is_none_or(|s| exported == s)
}

pub(crate) fn importer_symbol_matches(exported: &str, symbol: Option<&str>) -> bool {
    // A namespace import (`import * as Ns`) brings in every export, so it counts
    // as an importer for any requested symbol.
    exported == "*" || symbol.is_none_or(|s| exported == s)
}

pub(crate) fn filter_importers(files: &BTreeSet<String>, pred: fn(&str) -> bool) -> Vec<String> {
    files.iter().filter(|f| pred(f)).cloned().collect()
}

/// Exported `interface` / `type` names declared in the target file.
pub(crate) fn prop_type_names(path: &Path) -> Vec<String> {
    let source = std::fs::read_to_string(path).unwrap_or_default();
    let is_tsx = matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("tsx") | Some("jsx")
    );
    let Ok(symbols) = extract_symbols(&source, is_tsx) else {
        return Vec::new();
    };
    prop_type_names_from_symbols(&symbols)
}

pub(crate) fn prop_type_names_from_symbols(
    symbols: &crate::codebase::ts_symbols::FileSymbols,
) -> Vec<String> {
    let mut names: Vec<String> = symbols
        .exports
        .iter()
        .filter(|e| matches!(e.kind, ExportKind::Interface | ExportKind::TypeAlias))
        .map(|e| e.name.clone())
        .collect();
    names.sort();
    names.dedup();
    names
}

/// True when `path` resolves to the already-canonicalized `target`.
pub(crate) fn same_path(path: &Path, target: &Path) -> bool {
    path.canonicalize()
        .is_ok_and(|canonical| canonical == target)
}

fn basename(file: &str) -> &str {
    file.rsplit('/').next().unwrap_or(file)
}

pub(crate) fn is_story(file: &str) -> bool {
    basename(file).contains(".stories.")
}

pub(crate) fn is_test(file: &str) -> bool {
    let name = basename(file);
    name.contains(".test.") || name.contains(".spec.")
}
