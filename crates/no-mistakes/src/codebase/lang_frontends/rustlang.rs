use super::facts::{
    configured_roots, files_under, module_from_path, owning_package, LangFactMap, LangFileFacts,
};
use super::strip::strip_comments_keep_strings;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub(crate) fn collect_rust_facts(
    root: &Path,
    all_files: &[PathBuf],
    packages: &[String],
) -> LangFactMap {
    let roots = configured_roots(root, packages);
    let files = files_under(all_files, &roots, "rs");
    let mut facts = LangFactMap::default();
    for path in files {
        if let Some(file) = parse_rust_file(root, &path, &roots, packages) {
            facts.index_file(file);
        }
    }
    facts
}

fn parse_rust_file(
    root: &Path,
    path: &Path,
    roots: &[PathBuf],
    packages: &[String],
) -> Option<LangFileFacts> {
    let source = std::fs::read_to_string(path).ok()?;
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
    Some(LangFileFacts {
        path: path.to_path_buf(),
        package,
        module: src_root
            .as_ref()
            .and_then(|pkg| module_from_path(pkg, path)),
        imports: rust_imports(&text),
        declarations: extract_named(&text, rust_decl_re()),
        references: extract_named(&text, rust_ref_re()),
        route_handlers: Vec::new(),
        queue_enqueues: Vec::new(),
        queue_workers: Vec::new(),
    })
}

fn rust_imports(source: &str) -> Vec<String> {
    let mut imports = extract_named(source, rust_use_re());
    let prefixes: Vec<String> = imports
        .iter()
        .filter_map(|import| import.split('.').next().map(str::to_string))
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

fn rust_use_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?m)^\s*use\s+(?:crate|super|self)::([\w:]+)").expect("use"))
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
