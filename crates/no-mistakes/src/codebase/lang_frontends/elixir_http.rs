use regex::Regex;
use std::sync::OnceLock;

pub(super) fn extract_http_routes(source: &str) -> Vec<(String, String)> {
    let source = super::super::strip::mask_triple_quoted_strings(source);
    let mut prefixes: Vec<(usize, Option<String>)> = Vec::new();
    let mut routes = Vec::new();
    for line in source.lines() {
        let indent = line.len() - line.trim_start().len();
        let trimmed = line.trim();
        if trimmed == "end" {
            while prefixes.last().is_some_and(|(open, _)| *open >= indent) {
                prefixes.pop();
            }
            continue;
        }
        if let Some(scope) = parse_scope(trimmed) {
            prefixes.push((indent, scope));
            continue;
        }
        if prefixes.iter().any(|(_, prefix)| prefix.is_none()) {
            continue;
        }
        if let Some((path, handler)) = parse_verb(trimmed) {
            let prefix = prefixes
                .iter()
                .filter_map(|(_, prefix)| prefix.as_deref())
                .fold(None, |acc, part| {
                    Some(super::super::http_routes::join_route(acc.as_deref(), part))
                });
            routes.push((
                super::super::http_routes::join_route(prefix.as_deref(), &path),
                handler,
            ));
        }
    }
    routes.sort();
    routes.dedup();
    routes
}

fn parse_scope(trimmed: &str) -> Option<Option<String>> {
    if !(trimmed.starts_with("scope ") || trimmed.starts_with("scope(")) {
        return None;
    }
    phoenix_scope_re()
        .captures(trimmed)
        .map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .or(Some(None))
}

fn parse_verb(trimmed: &str) -> Option<(String, String)> {
    let cap = phoenix_route_re().captures(trimmed)?;
    Some((
        cap.get(1)?.as_str().to_string(),
        format!("{}.{}", cap.get(2)?.as_str(), cap.get(3)?.as_str()),
    ))
}

fn phoenix_scope_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"^scope\s+["']([^"']+)["']\s*(?:,|\s+do\b|$)"#).expect("phoenix scope")
    })
}

fn phoenix_route_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"^(?:get|post|put|patch|delete)\s+["']([^"']+)["']\s*,\s*([A-Z][A-Za-z0-9_.]*)\s*,\s*:([a-z_][A-Za-z0-9_]*)"#,
        )
        .expect("phoenix route")
    })
}

#[cfg(test)]
#[path = "elixir_http_tests.rs"]
mod tests;
