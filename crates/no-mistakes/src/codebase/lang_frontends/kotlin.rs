use super::facts::{configured_roots, files_under, owning_package, LangFactMap, LangFileFacts};
use super::strip::strip_comments_keep_strings;
#[path = "kotlin_http.rs"]
mod http;
use crate::codebase::ts_source::SourceStore;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub(crate) fn collect_kotlin_facts(
    root: &Path,
    all_files: &[PathBuf],
    packages: &[String],
    sources: &SourceStore,
) -> LangFactMap {
    let roots = configured_roots(root, packages);
    let files = files_under(all_files, &roots, "kt");
    super::facts::collect_files_parallel(files, |path| {
        parse_kotlin_file(path, &roots, packages, sources)
    })
}

fn parse_kotlin_file(
    path: &Path,
    roots: &[PathBuf],
    packages: &[String],
    sources: &SourceStore,
) -> Option<LangFileFacts> {
    let source = sources.read_path(path).ok()?;
    let text = strip_comments_keep_strings(&source);
    let symbols = super::strip::mask_strings(&text);
    let package = owning_package(path, roots, packages);
    let namespace = extract_package(&text);
    let declarations = extract_named(&symbols, kotlin_decl_re());
    let type_name = primary_type(
        &declarations,
        path.file_stem().and_then(|stem| stem.to_str()),
    );
    let module = match (namespace.as_deref(), type_name.as_deref()) {
        (Some(namespace), Some(name)) => Some(format!("{namespace}.{name}")),
        (None, Some(name)) => Some(name.to_string()),
        _ => None,
    };
    Some(LangFileFacts {
        path: path.to_path_buf(),
        package,
        module,
        imports: extract_kotlin_imports(&symbols),
        declarations,
        references: extract_named(&symbols, kotlin_ref_re()),
        route_handlers: http::extract_http_routes(&text),
        queue_enqueues: Vec::new(),
        queue_workers: Vec::new(),
        mods: Vec::new(),
    })
}

fn extract_package(source: &str) -> Option<String> {
    kotlin_package_re()
        .captures(source)
        .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
}

fn primary_type(declarations: &[String], file_stem: Option<&str>) -> Option<String> {
    file_stem
        .filter(|stem| declarations.iter().any(|name| name == *stem))
        .map(str::to_string)
        .or_else(|| declarations.first().cloned())
}

fn extract_kotlin_imports(source: &str) -> Vec<String> {
    let mut values: Vec<String> = kotlin_import_re()
        .captures_iter(source)
        .filter_map(|cap| {
            let path = cap.get(1)?.as_str();
            (!path.ends_with(".*") && !path.ends_with('.') && !path.contains('*'))
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

fn kotlin_package_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?m)^\s*package\s+([A-Za-z_][\w.]*)\s*;?").expect("pkg"))
}

fn kotlin_import_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?m)^\s*import\s+([A-Za-z_][\w.]*(?:\.\*)?)(?:\s+as\s+[A-Za-z_][\w]*)?\s*;?")
            .expect("import")
    })
}

fn kotlin_decl_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"\b(?:(?:public|private|protected|internal|open|abstract|final|sealed|data|enum|annotation|inner|value|fun)\s+)*\b(?:class|interface|object)\s+([A-Za-z_][A-Za-z0-9_]*)",
        )
        .expect("decl")
    })
}

fn kotlin_ref_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b([A-Z][A-Za-z0-9_]*)\b").expect("ref"))
}

#[cfg(test)]
#[path = "kotlin_tests.rs"]
mod tests;
