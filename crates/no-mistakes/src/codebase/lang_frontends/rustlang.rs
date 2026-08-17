use super::facts::{
    configured_roots, files_under, owning_package, rust_module_from_path, LangFactMap,
    LangFileFacts,
};
use super::strip::strip_comments_keep_strings;
#[path = "rust_use.rs"]
mod rust_use;
use crate::codebase::ts_source::SourceStore;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub(crate) fn collect_rust_facts(
    root: &Path,
    all_files: &[PathBuf],
    packages: &[String],
    sources: &SourceStore,
) -> LangFactMap {
    let roots = configured_roots(root, packages);
    let files = files_under(all_files, &roots, "rs");
    super::facts::collect_files_parallel(files, |path| {
        parse_rust_file(root, path, &roots, packages, sources)
    })
}

fn parse_rust_file(
    root: &Path,
    path: &Path,
    roots: &[PathBuf],
    packages: &[String],
    sources: &SourceStore,
) -> Option<LangFileFacts> {
    let source = sources.read_path(path).ok()?;
    let text = strip_comments_keep_strings(&source);
    let package = owning_package(path, roots, packages);
    let package_root = package
        .as_ref()
        .map(|name| crate::codebase::ts_resolver::normalize_path(&root.join(name)));
    let src_root = package_root.as_ref().map(|pkg| {
        let src = pkg.join("src");
        if src.is_dir() {
            src
        } else {
            pkg.clone()
        }
    });
    let module = src_root
        .as_ref()
        .and_then(|pkg| rust_module_from_path(pkg, path));
    Some(LangFileFacts {
        path: path.to_path_buf(),
        package,
        module: module.clone(),
        imports: rust_imports(&text, module.as_deref()),
        declarations: extract_named(&text, rust_decl_re()),
        references: extract_named(&text, rust_ref_re()),
        route_handlers: Vec::new(),
        queue_enqueues: Vec::new(),
        queue_workers: Vec::new(),
        mods: extract_named(&text, rust_mod_re()),
    })
}

fn rust_imports(source: &str, module: Option<&str>) -> Vec<String> {
    let mut imports = Vec::new();
    for cap in rust_use_re().captures_iter(source) {
        let kind = cap.get(1).map(|m| m.as_str()).unwrap_or("crate");
        let tree = cap.get(2).map(|m| m.as_str()).unwrap_or("");
        for item in rust_use::expand_rust_use(tree) {
            imports.push(rust_use::qualify_rust_use(
                kind,
                &item.replace("::", "."),
                module,
            ));
        }
    }
    let prefixes: Vec<String> = imports
        .iter()
        .flat_map(|import| rust_use::rust_path_prefixes(import))
        .collect();
    imports.extend(prefixes);
    imports.sort();
    imports.dedup();
    imports
}

fn extract_named(source: &str, re: &Regex) -> Vec<String> {
    let mut values: Vec<String> = re
        .captures_iter(source)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().replace("::", ".")))
        .collect();
    values.sort();
    values.dedup();
    values
}

fn rust_mod_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?m)^\s*(?:pub(?:\([^)]+\))?\s+)?mod\s+([A-Za-z_]\w*)\s*;").expect("mod")
    })
}

fn rust_use_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?m)^\s*(?:pub(?:\([^)]+\))?\s+)?use\s+(crate|super|self)::(.+?)\s*;")
            .expect("use")
    })
}

fn rust_decl_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?m)^\s*pub(?:\([^)]+\))?\s+(?:fn|struct|enum|trait|type|mod)\s+([A-Za-z_]\w*)",
        )
        .expect("decl")
    })
}

fn rust_ref_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b([A-Z][A-Za-z0-9_]*)\b").expect("ref"))
}
