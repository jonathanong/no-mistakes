use super::super::facts::{owning_package, LangFileFacts};
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub(super) fn yaml_route_facts(
    path: &Path,
    roots: &[PathBuf],
    apps: &[String],
    source: &str,
) -> LangFileFacts {
    LangFileFacts {
        path: path.to_path_buf(),
        package: owning_package(path, roots, apps),
        module: path
            .file_stem()
            .map(|name| name.to_string_lossy().into_owned()),
        route_handlers: extract_yaml_routes(source),
        ..LangFileFacts::default()
    }
}

pub(super) fn extract_php_routes(
    source: &str,
    laravel: bool,
    symfony: bool,
) -> Vec<(String, String)> {
    let mut routes = Vec::new();
    if laravel {
        routes.extend(extract_laravel_routes(source));
    }
    if symfony {
        routes.extend(extract_attribute_routes(source));
    }
    routes.sort();
    routes.dedup();
    routes
}

pub(super) fn extract_yaml_routes(source: &str) -> Vec<(String, String)> {
    let mut routes = extract_yaml_pairs(source, yaml_path_then_controller_re(), false);
    routes.extend(extract_yaml_pairs(
        source,
        yaml_controller_then_path_re(),
        true,
    ));
    routes.sort();
    routes.dedup();
    routes
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

fn extract_attribute_routes(source: &str) -> Vec<(String, String)> {
    attribute_route_re()
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

fn extract_yaml_pairs(source: &str, re: &Regex, controller_first: bool) -> Vec<(String, String)> {
    re.captures_iter(source)
        .filter_map(|cap| {
            let first = cap.get(1)?.as_str();
            let second = cap.get(2)?.as_str();
            let (path, handler) = if controller_first {
                (second, first)
            } else {
                (first, second)
            };
            Some((
                path.to_string(),
                handler.replace('\\', ".").replace("::", "."),
            ))
        })
        .collect()
}

fn laravel_route_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"Route::(?:get|post|put|patch|delete)\(\s*['"]([^'"]+)['"]\s*,\s*(?:\[([^\]]+)\]|(\\?[A-Za-z_][A-Za-z0-9_\\]*)::class)"#,
        )
        .expect("laravel route")
    })
}

fn attribute_route_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"#\[Route\(\s*(?:path:\s*)?['"]([^'"]+)['"][^)]*\)\s*\]\s*(?:(?:public|protected|private)\s+)?(?:function\s+(\w+)|(?:final\s+|abstract\s+|readonly\s+)*class\s+(\w+))"#,
        )
        .expect("symfony route")
    })
}

fn yaml_path_then_controller_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"(?m)^\s*path:\s*['"]?(/[^'"\s]+)['"]?\s*\n(?:[ \t].*\n)*?[ \t]*controller:\s*['"]?([A-Za-z_\\][A-Za-z0-9_\\:]*)"#,
        )
        .expect("yaml path controller")
    })
}

fn yaml_controller_then_path_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"(?m)^\s*controller:\s*['"]?([A-Za-z_\\][A-Za-z0-9_\\:]*)['"]?\s*\n(?:[ \t].*\n)*?[ \t]*path:\s*['"]?(/[^'"\s]+)"#,
        )
        .expect("yaml controller path")
    })
}

#[cfg(test)]
#[path = "php_http_tests.rs"]
mod tests;
