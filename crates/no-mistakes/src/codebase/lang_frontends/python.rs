use super::facts::{configured_roots, files_under, owning_package, LangFactMap, LangFileFacts};
use super::strip::strip_comments_keep_strings;
#[path = "python_imports.rs"]
mod imports;
use imports::{extract_python_imports, python_module};
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub(crate) fn collect_python_facts(
    root: &Path,
    all_files: &[PathBuf],
    packages: &[String],
) -> LangFactMap {
    let roots = configured_roots(root, packages);
    let files = files_under(all_files, &roots, "py");
    super::facts::collect_files_parallel(files, |path| {
        parse_python_file(root, path, &roots, packages)
    })
}

fn parse_python_file(
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
    let module = python_module(package.as_deref(), package_root.as_deref(), path);
    let imports = extract_python_imports(&text, path, package.as_deref(), package_root.as_deref());
    Some(LangFileFacts {
        path: path.to_path_buf(),
        package,
        module,
        imports,
        declarations: extract_named(&text, python_decl_re()),
        references: extract_named(&text, python_ref_re()),
        route_handlers: extract_pairs(&text, django_route_re()),
        queue_enqueues: extract_named(&text, celery_enqueue_re()),
        queue_workers: extract_celery_workers(&text),
        mods: Vec::new(),
    })
}

fn extract_celery_workers(source: &str) -> Vec<String> {
    let mut names = extract_named(source, celery_named_task_re());
    names.extend(extract_named(source, celery_fn_task_re()));
    names.sort();
    names.dedup();
    names
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

fn extract_pairs(source: &str, re: &Regex) -> Vec<(String, String)> {
    re.captures_iter(source)
        .filter_map(|cap| {
            Some((
                cap.get(1)?.as_str().to_string(),
                cap.get(2)?.as_str().to_string(),
            ))
        })
        .collect()
}

fn python_decl_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?m)^\s*(?:async\s+)?(?:def|class)\s+([A-Za-z_]\w*)").expect("decl")
    })
}

fn python_ref_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b([A-Z][A-Za-z0-9_]+)\b").expect("ref"))
}

fn django_route_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"\b(?:path|re_path)\(\s*["']([^"']+)["']\s*,\s*([A-Za-z_][\w.]*)"#)
            .expect("django")
    })
}

fn celery_enqueue_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b([A-Za-z_]\w*)\.(?:delay|apply_async)\s*\(").expect("enqueue"))
}

fn celery_named_task_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"@(?:shared_task|[\w.]+\.task)\([^)]*name\s*=\s*["']([^"']+)["']"#)
            .expect("named")
    })
}

fn celery_fn_task_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?m)@(?:shared_task|[\w.]+\.task)(?:\([^)]*\))?\s*\n\s*(?:async\s+)?def\s+([A-Za-z_]\w*)")
            .expect("fn task")
    })
}
