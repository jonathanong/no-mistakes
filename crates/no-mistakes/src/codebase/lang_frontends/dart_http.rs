use regex::Regex;
use std::sync::OnceLock;

pub(crate) fn extract_http_paths(source: &str) -> Vec<String> {
    let source = super::super::strip::strip_comments_keep_strings(source);
    let mut values: Vec<String> = dart_uri_parse_re()
        .captures_iter(&source)
        .filter_map(|cap| static_path(cap.get(1)?.as_str()))
        .collect();
    values.extend(
        dart_http_verb_re()
            .captures_iter(&source)
            .filter_map(|cap| static_path(cap.get(1)?.as_str())),
    );
    values.sort();
    values.dedup();
    values
}

fn static_path(raw: &str) -> Option<String> {
    (!raw.contains('$')).then(|| raw).and_then(normalize_path)
}

fn normalize_path(raw: &str) -> Option<String> {
    let path = hosted_path(raw).unwrap_or(raw);
    let path = path.split(['?', '#']).next().unwrap_or(path);
    path.starts_with('/').then(|| path.to_string())
}

fn hosted_path(raw: &str) -> Option<&str> {
    let rest = raw.split_once("://")?.1;
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    rest[end..].starts_with('/').then_some(&rest[end..])
}

fn dart_uri_parse_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"\bUri\.parse\(\s*r?['"]([^'"]+)['"]"#).expect("uri"))
}

fn dart_http_verb_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"\bhttp\.(?:get|post|put|patch|delete)\(\s*r?['"]([^'"]+)['"]"#).expect("http")
    })
}

#[cfg(test)]
#[path = "dart_http_tests.rs"]
mod tests;
