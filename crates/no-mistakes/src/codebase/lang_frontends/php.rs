use super::facts::{configured_roots, files_under, owning_package, LangFactMap, LangFileFacts};
use super::strip::strip_comments_keep_strings;
#[path = "php_queue.rs"]
mod queue;
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
    let queue_workers = if laravel && queue::php_should_queue_re().is_match(&text) {
        queue::laravel_queue_identities(&classes)
    } else {
        Vec::new()
    };
    Some(LangFileFacts {
        path: path.to_path_buf(),
        package: owning_package(path, roots, apps),
        module: classes.first().cloned().or_else(|| {
            path.file_stem()
                .map(|name| name.to_string_lossy().into_owned())
        }),
        imports: {
            let mut imports = extract_php_uses(&text);
            imports.extend(queue::extract_php_requires(&text));
            imports
        },
        declarations: classes,
        references: extract_named(&text, php_use_re()),
        route_handlers: if laravel {
            extract_laravel_routes(&text)
        } else {
            Vec::new()
        },
        queue_enqueues: if laravel {
            queue::extract_laravel_dispatches(&text)
        } else {
            Vec::new()
        },
        queue_workers,
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

pub(super) fn extract_named(source: &str, re: &Regex) -> Vec<String> {
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
            let handler = cap.get(2).or_else(|| cap.get(3))?.as_str();
            Some((
                cap.get(1)?.as_str().into(),
                handler.replace('\\', ".").trim_start_matches('.').into(),
            ))
        })
        .collect()
}

fn php_namespace_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?m)^\s*namespace\s+([A-Za-z_\\][A-Za-z0-9_\\]*)").expect("ns"))
}

pub(super) fn extract_php_uses(source: &str) -> Vec<String> {
    let mut imports = extract_named(source, php_use_re());
    for cap in php_alias_use_re().captures_iter(source) {
        let path = cap
            .get(1)
            .map(|m| m.as_str().replace('\\', "."))
            .unwrap_or_default();
        if let Some(alias) = cap.get(2).map(|m| m.as_str()) {
            imports.push(format!("{alias}={path}"));
        }
    }
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
                let qualified = format!("{prefix}.{ident}");
                imports.push(qualified.clone());
                if let Some(alias) = member
                    .split_whitespace()
                    .skip_while(|token| !token.eq_ignore_ascii_case("as"))
                    .nth(1)
                {
                    imports.push(format!("{alias}={qualified}"));
                }
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

fn php_alias_use_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?m)^\s*use\s+([A-Za-z_\\][A-Za-z0-9_\\]*)\s+as\s+([A-Za-z_][A-Za-z0-9_]*)")
            .expect("alias use")
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
            r"(?m)^\s*(?:final\s+|abstract\s+|readonly\s+)*(?:class|interface|trait|enum)\s+([A-Za-z_][A-Za-z0-9_]*)",
        )
        .expect("class")
    })
}

fn laravel_route_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"Route::(?:get|post|put|patch|delete)\(\s*['"]([^'"]+)['"]\s*,\s*(?:\[([^\]]+)\]|(\\?[A-Za-z_][A-Za-z0-9_\\]*)::class)"#,
        )
        .expect("route")
    })
}
