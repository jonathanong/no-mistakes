use super::facts::{configured_roots, files_under, owning_package, LangFactMap, LangFileFacts};
use super::strip::strip_comments_keep_strings;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub(crate) fn collect_php_facts(
    root: &Path,
    all_files: &[PathBuf],
    apps: &[String],
    framework: Option<&str>,
) -> LangFactMap {
    let roots = configured_roots(root, apps);
    let mut files = files_under(all_files, &roots, "php");
    for root in &roots {
        let composer = root.join("composer.json");
        if all_files.iter().any(|path| path == &composer) {
            files.push(composer);
        }
    }
    let laravel = framework.is_some_and(|name| name.eq_ignore_ascii_case("laravel"));
    super::facts::collect_files_parallel(files, |path| parse_php_file(path, &roots, apps, laravel))
}

fn parse_php_file(
    path: &Path,
    roots: &[PathBuf],
    apps: &[String],
    laravel: bool,
) -> Option<LangFileFacts> {
    let source = std::fs::read_to_string(path).ok()?;
    let text = strip_comments_keep_strings(&source);
    let classes = php_classes(&text);
    Some(LangFileFacts {
        path: path.to_path_buf(),
        package: owning_package(path, roots, apps),
        module: classes.first().cloned(),
        imports: extract_php_uses(&text),
        declarations: classes,
        references: extract_named(&text, php_use_re()),
        route_handlers: if laravel {
            extract_laravel_routes(&text)
        } else {
            Vec::new()
        },
        queue_enqueues: if laravel {
            extract_named(&text, laravel_dispatch_re())
        } else {
            Vec::new()
        },
        queue_workers: if laravel && php_should_queue_re().is_match(&text) {
            extract_named(&text, php_class_re())
        } else {
            Vec::new()
        },
        mods: Vec::new(),
    })
}

fn php_classes(source: &str) -> Vec<String> {
    let namespace = php_namespace_re()
        .captures(source)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().replace('\\', "."));
    let mut names = extract_named(source, php_class_re());
    if let Some(namespace) = namespace {
        names.extend(
            names
                .clone()
                .into_iter()
                .map(|name| format!("{namespace}.{name}")),
        );
    }
    names.sort();
    names.dedup();
    names
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

fn extract_laravel_routes(source: &str) -> Vec<(String, String)> {
    laravel_route_re()
        .captures_iter(source)
        .filter_map(|cap| {
            Some((
                cap.get(1)?.as_str().to_string(),
                cap.get(2)
                    .or_else(|| cap.get(3))?
                    .as_str()
                    .replace('\\', "."),
            ))
        })
        .collect()
}

fn php_namespace_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?m)^\s*namespace\s+([A-Za-z_\\][A-Za-z0-9_\\]*)").expect("ns"))
}

fn extract_php_uses(source: &str) -> Vec<String> {
    let mut imports = extract_named(source, php_use_re());
    for cap in php_group_use_re().captures_iter(source) {
        let prefix = cap
            .get(1)
            .map(|m| m.as_str().replace('\\', "."))
            .unwrap_or_default();
        for member in cap.get(2).map(|m| m.as_str()).unwrap_or("").split(',') {
            let ident = member
                .split_whitespace()
                .take_while(|token| !token.eq_ignore_ascii_case("as"))
                .next()
                .unwrap_or("")
                .trim();
            if !ident.is_empty() {
                imports.push(format!("{prefix}.{ident}"));
            }
        }
    }
    imports.sort();
    imports.dedup();
    imports
}

fn php_group_use_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?m)^\s*use\s+([A-Za-z_\\][A-Za-z0-9_\\]*)\\\{([^}]+)\}").expect("group use")
    })
}

fn php_use_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?m)^\s*use\s+([A-Za-z_\\][A-Za-z0-9_\\]*)").expect("use"))
}

fn php_class_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?m)^\s*(?:final\s+|abstract\s+)?(?:class|interface|trait|enum)\s+([A-Za-z_][A-Za-z0-9_]*)",
        )
        .expect("class")
    })
}

fn laravel_route_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"Route::(?:get|post|put|patch|delete)\(\s*['"]([^'"]+)['"]\s*,\s*(?:\[([^\]]+)\]|([A-Za-z_\\][A-Za-z0-9_\\]*)::class)"#,
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
    RE.get_or_init(|| Regex::new(r"\bimplements\s+[^{;]*\bShouldQueue\b").expect("shouldqueue"))
}
