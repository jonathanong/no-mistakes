use super::facts::{configured_roots, files_under, owning_package, LangFactMap, LangFileFacts};
use super::strip::strip_comments_keep_strings;
#[path = "elixir_http.rs"]
mod http;
use crate::codebase::ts_source::SourceStore;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub(crate) fn collect_elixir_facts(
    root: &Path,
    all_files: &[PathBuf],
    apps: &[String],
    sources: &SourceStore,
) -> LangFactMap {
    let roots = configured_roots(root, apps);
    let mut files = files_under(all_files, &roots, "ex");
    files.extend(files_under(all_files, &roots, "exs"));
    files.retain(|path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_none_or(|name| name != "mix.exs")
    });
    super::facts::collect_files_parallel(files, |path| {
        parse_elixir_file(path, &roots, apps, sources)
    })
}

fn parse_elixir_file(
    path: &Path,
    roots: &[PathBuf],
    apps: &[String],
    sources: &SourceStore,
) -> Option<LangFileFacts> {
    let source = sources.read_path(path).ok()?;
    let text = strip_comments_keep_strings(&source);
    let symbols = super::strip::mask_strings(&text);
    let mut declarations = extract_named(&symbols, elixir_defmodule_re());
    let type_name = primary_module(
        &declarations,
        path.file_stem().and_then(|stem| stem.to_str()),
    );
    let tails: Vec<String> = declarations
        .iter()
        .filter_map(|name| name.rsplit_once('.').map(|(_, last)| last.to_string()))
        .collect();
    declarations.extend(tails);
    declarations.sort();
    declarations.dedup();
    Some(LangFileFacts {
        path: path.to_path_buf(),
        package: owning_package(path, roots, apps),
        module: type_name,
        imports: extract_elixir_imports(&symbols),
        declarations,
        references: extract_named(&symbols, elixir_ref_re()),
        route_handlers: if elixir_test_tree(path, roots) {
            Vec::new()
        } else {
            http::extract_http_routes(&text)
        },
        queue_enqueues: Vec::new(),
        queue_workers: Vec::new(),
        mods: Vec::new(),
    })
}

pub(super) fn primary_module(declarations: &[String], file_stem: Option<&str>) -> Option<String> {
    let pascal = file_stem.map(elixir_pascal_case);
    pascal
        .as_deref()
        .and_then(|stem| {
            declarations
                .iter()
                .find(|name| name == &stem || name.rsplit('.').next() == Some(stem))
                .cloned()
        })
        .or_else(|| declarations.first().cloned())
}

fn elixir_test_tree(path: &Path, roots: &[PathBuf]) -> bool {
    roots.iter().any(|root| {
        path.strip_prefix(root).is_ok_and(|rel| {
            rel.components()
                .any(|seg| seg.as_os_str() == std::ffi::OsStr::new("test"))
        })
    })
}

fn elixir_pascal_case(stem: &str) -> String {
    stem.split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut out = String::new();
            for (i, ch) in part.chars().enumerate() {
                if i == 0 {
                    out.extend(ch.to_uppercase());
                } else {
                    out.push(ch);
                }
            }
            out
        })
        .collect()
}

pub(super) fn extract_elixir_imports(source: &str) -> Vec<String> {
    let mut values: Vec<String> = elixir_import_re()
        .captures_iter(source)
        .filter_map(|cap| {
            let matched = cap.get(1)?;
            let path = matched.as_str();
            let rest = source.get(matched.end()..).unwrap_or("");
            let rest = rest.trim_start();
            (!path.contains('{')
                && !path.contains('*')
                && !rest.starts_with('{')
                && !rest.starts_with(".{"))
            .then(|| path.to_string())
        })
        .collect();
    values.sort();
    values.dedup();
    values
}

fn extract_named(source: &str, re: &Regex) -> Vec<String> {
    let mut values: Vec<String> = re
        .captures_iter(source)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect();
    values.sort();
    values.dedup();
    values
}

fn elixir_defmodule_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"\bdefmodule\s+([A-Z][A-Za-z0-9_]*(?:\.[A-Z][A-Za-z0-9_]*)*)")
            .expect("defmodule")
    })
}

fn elixir_import_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?m)^\s*(?:alias|import|use)\s+([A-Z][A-Za-z0-9_]*(?:\.[A-Z][A-Za-z0-9_]*)*)")
            .expect("import")
    })
}

fn elixir_ref_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b([A-Z][A-Za-z0-9_]*)\b").expect("ref"))
}

#[cfg(test)]
#[path = "elixir_tests.rs"]
mod tests;
