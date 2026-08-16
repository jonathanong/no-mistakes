use super::facts::{
    configured_roots, files_under, module_from_path, owning_package, LangFactMap, LangFileFacts,
};
use super::strip::strip_comments_keep_strings;
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
    let mut facts = LangFactMap::default();
    for path in files {
        if let Some(file) = parse_python_file(root, &path, &roots, packages) {
            facts.index_file(file);
        }
    }
    facts
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
    let module = package_root
        .as_ref()
        .and_then(|pkg| module_from_path(pkg, path));
    Some(LangFileFacts {
        path: path.to_path_buf(),
        package,
        module,
        imports: extract_python_imports(&text, path, package_root.as_deref()),
        declarations: extract_named(&text, python_decl_re()),
        references: extract_named(&text, python_ref_re()),
        route_handlers: extract_pairs(&text, django_route_re()),
        queue_enqueues: extract_named(&text, celery_enqueue_re()),
        queue_workers: extract_celery_workers(&text),
    })
}

fn extract_python_imports(source: &str, path: &Path, package_root: Option<&Path>) -> Vec<String> {
    let mut imports = extract_named(source, python_import_re());
    for cap in python_from_re().captures_iter(source) {
        let Some(module) = cap.get(1).map(|m| m.as_str()) else {
            continue;
        };
        if let Some(resolved) = resolve_relative(module, path, package_root) {
            imports.push(resolved);
        } else if !module.starts_with('.') {
            imports.push(module.to_string());
        }
    }
    imports.sort();
    imports.dedup();
    imports
}

fn resolve_relative(module: &str, path: &Path, package_root: Option<&Path>) -> Option<String> {
    let dots = module.chars().take_while(|ch| *ch == '.').count();
    if dots == 0 {
        return None;
    }
    let rest = module[dots..].trim_matches('.');
    let mut dir = path.parent()?.to_path_buf();
    for _ in 1..dots {
        dir = dir.parent()?.to_path_buf();
    }
    let package_root = package_root?;
    let target = if rest.is_empty() {
        dir
    } else {
        dir.join(rest.replace('.', std::path::MAIN_SEPARATOR_STR))
    };
    module_from_path(package_root, &target.with_extension("py"))
        .or_else(|| module_from_path(package_root, &target.join("__init__.py")))
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

fn python_import_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?m)^\s*import\s+([A-Za-z_][\w.]*)").expect("python import"))
}

fn python_from_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?m)^\s*from\s+(\.+(?:[A-Za-z_][\w.]*)?|[A-Za-z_][\w.]*)\s+import")
            .expect("from")
    })
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
