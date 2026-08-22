use super::facts::{configured_roots, files_under, owning_package, LangFactMap, LangFileFacts};
use super::strip::strip_comments_keep_strings;
#[path = "dart_http.rs"]
mod http;
use crate::codebase::ts_source::SourceStore;
pub(crate) use http::extract_http_paths;
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub(crate) fn collect_dart_facts(
    root: &Path,
    all_files: &[PathBuf],
    packages: &[String],
    sources: &SourceStore,
) -> LangFactMap {
    let roots = configured_roots(root, packages);
    let names: HashMap<PathBuf, String> = roots
        .iter()
        .filter_map(|package_root| {
            read_pubspec_name(package_root, sources).map(|name| (package_root.clone(), name))
        })
        .collect();
    let files = files_under(all_files, &roots, "dart");
    super::facts::collect_files_parallel(files, |path| {
        parse_dart_file(path, &roots, packages, &names, sources)
    })
}

fn parse_dart_file(
    path: &Path,
    roots: &[PathBuf],
    packages: &[String],
    names: &HashMap<PathBuf, String>,
    sources: &SourceStore,
) -> Option<LangFileFacts> {
    let source = sources.read_path(path).ok()?;
    let text = strip_comments_keep_strings(&source);
    let symbols = super::strip::mask_strings(&text);
    let package = owning_package(path, roots, packages);
    let package_root = roots
        .iter()
        .zip(packages.iter())
        .filter(|(root, _)| path.starts_with(root))
        .max_by_key(|(root, _)| root.components().count())
        .map(|(root, _)| root.clone());
    let package_name = package_root
        .as_ref()
        .and_then(|root| names.get(root))
        .cloned();
    let module = package_root
        .as_ref()
        .and_then(|root| dart_module(path, root, package_name.as_deref()));
    Some(LangFileFacts {
        path: path.to_path_buf(),
        package,
        module,
        imports: extract_dart_imports(
            &text,
            path,
            package_root.as_deref(),
            package_name.as_deref(),
        ),
        declarations: extract_named(&symbols, dart_decl_re()),
        references: extract_named(&symbols, dart_ref_re()),
        route_handlers: Vec::new(),
        queue_enqueues: Vec::new(),
        queue_workers: Vec::new(),
        mods: Vec::new(),
    })
}

fn dart_module(path: &Path, package_root: &Path, package_name: Option<&str>) -> Option<String> {
    let rel = path.strip_prefix(package_root).ok()?;
    let rel = rel.to_string_lossy().replace('\\', "/");
    if let Some(name) = package_name {
        if let Some(rest) = rel.strip_prefix("lib/") {
            return Some(format!("package:{name}/{rest}"));
        }
    }
    Some(rel)
}

pub(super) fn extract_dart_imports(
    source: &str,
    path: &Path,
    package_root: Option<&Path>,
    package_name: Option<&str>,
) -> Vec<String> {
    let mut values: Vec<String> = dart_import_re()
        .captures_iter(source)
        .filter_map(|cap| {
            let uri = cap.get(1)?.as_str();
            resolve_dart_uri(uri, path, package_root, package_name)
        })
        .collect();
    values.sort();
    values.dedup();
    values
}

fn resolve_dart_uri(
    uri: &str,
    path: &Path,
    package_root: Option<&Path>,
    package_name: Option<&str>,
) -> Option<String> {
    if uri.starts_with("dart:") || uri.contains('*') {
        return None;
    }
    if uri.starts_with("package:") {
        return Some(uri.to_string());
    }
    let dir = path.parent()?;
    let resolved = crate::codebase::ts_resolver::normalize_path(&dir.join(uri));
    package_root.and_then(|root| dart_module(&resolved, root, package_name))
}

fn read_pubspec_name(package_root: &Path, sources: &SourceStore) -> Option<String> {
    let path = package_root.join("pubspec.yaml");
    let source = sources.read_path(&path).ok()?;
    pubspec_name_re()
        .captures(&source)
        .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
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

fn dart_import_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?m)^\s*(?:import|export|part(?:\s+of)?)\s+['"]([^'"]+)['"]"#)
            .expect("import")
    })
}

fn dart_decl_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"\b(?:class|mixin|enum|extension(?:\s+type)?|typedef)\s+([A-Z][A-Za-z0-9_]*)")
            .expect("decl")
    })
}

fn dart_ref_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b([A-Z][A-Za-z0-9_]*)\b").expect("ref"))
}

fn pubspec_name_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?m)^\s*name:\s*['"]?([A-Za-z_][\w]*)['"]?\s*(?:#.*)?$"#).expect("pubspec")
    })
}

#[cfg(test)]
#[path = "dart_tests.rs"]
mod tests;
