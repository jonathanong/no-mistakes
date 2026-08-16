use super::facts::{configured_roots, files_under, owning_package, LangFactMap, LangFileFacts};
use super::strip::strip_comments_keep_strings;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub(crate) fn collect_php_facts(
    root: &Path,
    all_files: &[PathBuf],
    apps: &[String],
) -> LangFactMap {
    let roots = configured_roots(root, apps);
    let files = files_under(all_files, &roots, "php");
    let mut facts = LangFactMap::default();
    for path in files {
        if let Some(file) = parse_php_file(&path, &roots, apps) {
            facts.index_file(file);
        }
    }
    facts
}

fn parse_php_file(path: &Path, roots: &[PathBuf], apps: &[String]) -> Option<LangFileFacts> {
    let source = std::fs::read_to_string(path).ok()?;
    let text = strip_comments_keep_strings(&source);
    Some(LangFileFacts {
        path: path.to_path_buf(),
        package: owning_package(path, roots, apps),
        module: extract_named(&text, php_class_re()).into_iter().next(),
        imports: extract_named(&text, php_use_re()),
        declarations: extract_named(&text, php_class_re()),
        references: extract_named(&text, php_class_re()),
        route_handlers: extract_pairs(&text, laravel_route_re()),
        queue_enqueues: extract_named(&text, laravel_dispatch_re()),
        queue_workers: if php_should_queue_re().is_match(&text) {
            extract_named(&text, php_class_re())
        } else {
            Vec::new()
        },
    })
}

fn extract_named(source: &str, re: &Regex) -> Vec<String> {
    let mut values: Vec<String> = re
        .captures_iter(source)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().replace('\\', ".")))
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
                cap.get(2)?.as_str().replace('\\', "."),
            ))
        })
        .collect()
}

fn php_use_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?m)^\s*use\s+([A-Za-z_\\][A-Za-z0-9_\\]*)").expect("use"))
}

fn php_class_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?m)^\s*(?:final\s+|abstract\s+)?class\s+([A-Za-z_][A-Za-z0-9_]*)")
            .expect("class")
    })
}

fn laravel_route_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"Route::(?:get|post|put|patch|delete)\(\s*['"]([^'"]+)['"]\s*,\s*\[([^\]]+)\]"#,
        )
        .expect("route")
    })
}

fn laravel_dispatch_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b([A-Za-z_][A-Za-z0-9_]*)::dispatch\s*\(").expect("dispatch"))
}

fn php_should_queue_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\bimplements\s+ShouldQueue\b").expect("shouldqueue"))
}
