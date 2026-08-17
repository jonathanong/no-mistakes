use super::facts::{configured_roots, files_under, owning_package, LangFactMap, LangFileFacts};
use super::strip::strip_comments_keep_strings;
#[path = "python_imports.rs"]
mod imports;
use imports::{extract_python_imports, python_module};
#[path = "python_http.rs"]
mod http;

#[cfg(test)]
#[path = "python_imports_tests.rs"]
mod tests;
use crate::codebase::ts_source::SourceStore;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub(crate) fn collect_python_facts(
    root: &Path,
    all_files: &[PathBuf],
    packages: &[String],
    sources: &SourceStore,
) -> LangFactMap {
    let roots = configured_roots(root, packages);
    let files = files_under(all_files, &roots, "py");
    super::facts::collect_files_parallel(files, |path| {
        parse_python_file(root, path, &roots, packages, sources)
    })
}

fn parse_python_file(
    root: &Path,
    path: &Path,
    roots: &[PathBuf],
    packages: &[String],
    sources: &SourceStore,
) -> Option<LangFileFacts> {
    let source = sources.read_path(path).ok()?;
    let text = strip_comments_keep_strings(&source);
    let symbols = super::strip::mask_strings(&text);
    let package = owning_package(path, roots, packages);
    let package_root = package
        .as_ref()
        .map(|name| crate::codebase::ts_resolver::normalize_path(&root.join(name)));
    let module = python_module(package.as_deref(), package_root.as_deref(), path);
    let imports = extract_python_imports(&text, path, package.as_deref(), package_root.as_deref());
    let queue_enqueues = extract_celery_enqueues(&text, &imports);
    let queue_workers = extract_celery_workers(&text, module.as_deref());
    Some(LangFileFacts {
        path: path.to_path_buf(),
        package,
        module,
        imports,
        declarations: extract_named(&symbols, python_decl_re()),
        references: extract_named(&symbols, python_ref_re()),
        route_handlers: http::extract_http_routes(&text),
        queue_enqueues,
        queue_workers,
        mods: Vec::new(),
    })
}

fn extract_celery_workers(source: &str, module: Option<&str>) -> Vec<String> {
    let mut names = extract_named(source, celery_named_task_re());
    for name in extract_named(source, celery_fn_task_re()) {
        names.push(match module {
            Some(module) => format!("{module}.{name}"),
            None => name,
        });
    }
    names.sort();
    names.dedup();
    names
}

fn extract_celery_enqueues(source: &str, imports: &[String]) -> Vec<String> {
    let mut names = extract_named(source, celery_enqueue_re());
    let extras: Vec<String> = names
        .iter()
        .flat_map(|name| {
            imports.iter().filter_map(move |import| {
                let target = import
                    .split_once('=')
                    .map(|(_, target)| target)
                    .unwrap_or(import);
                if target == name || target.ends_with(&format!(".{name}")) {
                    Some(target.to_string())
                } else {
                    None
                }
            })
        })
        .collect();
    names.extend(extras);
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
