use regex::Regex;
use std::sync::OnceLock;

pub(super) fn extract_http_routes(source: &str) -> Vec<(String, String)> {
    let source = super::super::strip::mask_triple_quoted_strings(source);
    let mut routes = extract_django_routes(&source);
    routes.extend(extract_decorator_routes(&source));
    routes.sort();
    routes.dedup();
    routes
}

fn extract_django_routes(source: &str) -> Vec<(String, String)> {
    django_route_re()
        .captures_iter(source)
        .filter_map(|cap| {
            Some((
                cap.get(1)?.as_str().to_string(),
                cap.get(2).or_else(|| cap.get(3))?.as_str().to_string(),
            ))
        })
        .collect()
}

fn extract_decorator_routes(source: &str) -> Vec<(String, String)> {
    flask_fastapi_route_re()
        .captures_iter(source)
        .filter_map(|cap| {
            Some((
                cap.get(1)?.as_str().to_string(),
                cap.get(2)?.as_str().to_string(),
            ))
        })
        .collect()
}

fn django_route_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"\b(?:path|re_path)\(\s*["']([^"']*)["']\s*,\s*(?:include\(\s*["']([^"']+)["']|([A-Za-z_][\w.]*))"#,
        )
        .expect("django")
    })
}

fn flask_fastapi_route_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"(?s)@(?:[\w.]+\.)?(?:route|get|post|put|patch|delete|head|options)\(\s*["']([^"']+)["'][^)]*\)\s*(?:async\s+)?def\s+([A-Za-z_]\w*)"#,
        )
        .expect("flask fastapi")
    })
}

#[cfg(test)]
#[path = "python_http_tests.rs"]
mod tests;
