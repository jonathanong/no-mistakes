use regex::Regex;
use std::sync::OnceLock;

pub(super) fn extract_http_routes(source: &str) -> Vec<(String, String)> {
    let source = super::super::strip::mask_triple_quoted_strings(source);
    let mut routes = phoenix_route_re()
        .captures_iter(&source)
        .filter_map(|cap| {
            let path = cap.get(1)?.as_str();
            let controller = cap.get(2)?.as_str();
            let action = cap.get(3)?.as_str();
            Some((normalize_path(path), format!("{controller}.{action}")))
        })
        .collect::<Vec<_>>();
    routes.sort();
    routes.dedup();
    routes
}

fn normalize_path(raw: &str) -> String {
    if raw.starts_with('/') {
        raw.to_string()
    } else {
        format!("/{raw}")
    }
}

fn phoenix_route_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"(?m)^\s*(?:get|post|put|patch|delete)\s+["']([^"']+)["']\s*,\s*([A-Z][A-Za-z0-9_.]*)\s*,\s*:([a-z_][A-Za-z0-9_]*)"#,
        )
        .expect("phoenix route")
    })
}

#[cfg(test)]
#[path = "elixir_http_tests.rs"]
mod tests;
