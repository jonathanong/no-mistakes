use super::facts::{configured_roots, files_under, owning_package, LangFactMap, LangFileFacts};
use super::strip::strip_comments_keep_strings;
#[path = "java_http.rs"]
mod http;
use crate::codebase::ts_source::SourceStore;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub(crate) fn collect_java_facts(
    root: &Path,
    all_files: &[PathBuf],
    packages: &[String],
    sources: &SourceStore,
) -> LangFactMap {
    let roots = configured_roots(root, packages);
    let files = files_under(all_files, &roots, "java");
    super::facts::collect_files_parallel(files, |path| {
        parse_java_file(path, &roots, packages, sources)
    })
}

fn parse_java_file(
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
    let declarations = extract_named(&symbols, java_decl_re());
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
        imports: extract_java_imports(&symbols),
        declarations,
        references: extract_named(&symbols, java_ref_re()),
        route_handlers: http::extract_http_routes(&text),
        queue_enqueues: Vec::new(),
        queue_workers: Vec::new(),
        mods: Vec::new(),
    })
}

fn extract_package(source: &str) -> Option<String> {
    java_package_re()
        .captures(source)
        .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
}

fn primary_type(declarations: &[String], file_stem: Option<&str>) -> Option<String> {
    file_stem
        .filter(|stem| declarations.iter().any(|name| name == *stem))
        .map(str::to_string)
        .or_else(|| declarations.first().cloned())
}

fn extract_java_imports(source: &str) -> Vec<String> {
    let mut values: Vec<String> = java_import_re()
        .captures_iter(source)
        .filter_map(|cap| {
            if cap.get(1).is_some() {
                return None;
            }
            let path = cap.get(2)?.as_str();
            (!path.ends_with(".*")).then(|| path.to_string())
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

fn java_package_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?m)^\s*package\s+([A-Za-z_][\w.]*)\s*;").expect("pkg"))
}

fn java_import_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?m)^\s*import\s+(static\s+)?([A-Za-z_][\w.]*(?:\.\*)?)\s*;").expect("import")
    })
}

fn java_decl_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"\b(?:public|protected|private|abstract|final|sealed|static|\s)*\b(?:class|interface|enum|record)\s+([A-Za-z_][A-Za-z0-9_]*)",
        )
        .expect("decl")
    })
}

fn java_ref_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b([A-Z][A-Za-z0-9_]*)\b").expect("ref"))
}

#[cfg(test)]
#[path = "java_tests.rs"]
mod tests;
