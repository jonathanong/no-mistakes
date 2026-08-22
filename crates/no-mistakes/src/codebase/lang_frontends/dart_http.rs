use regex::Regex;
use std::sync::OnceLock;

pub(crate) fn extract_http_paths(source: &str) -> Vec<String> {
    let mut values: Vec<String> = dart_uri_parse_re()
        .captures_iter(source)
        .filter_map(|cap| normalize_path(cap.get(1)?.as_str()))
        .collect();
    values.extend(
        dart_http_verb_re()
            .captures_iter(source)
            .filter_map(|cap| normalize_path(cap.get(1)?.as_str())),
    );
    values.sort();
    values.dedup();
    values
}

fn normalize_path(raw: &str) -> Option<String> {
    let path = if let Some(host) = raw.find("://").and_then(|index| raw[index + 3..].find('/')) {
        let start = raw.find("://")? + 3 + host;
        &raw[start..]
    } else {
        raw
    };
    let path = path.split('?').next().unwrap_or(path);
    path.starts_with('/').then(|| path.to_string())
}

fn dart_uri_parse_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"Uri\.parse\(\s*['"]([^'"]+)['"]"#).expect("uri"))
}

fn dart_http_verb_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"http\.(?:get|post|put|patch|delete)\(\s*['"]([^'"]+)['"]"#).expect("http")
    })
}

#[cfg(test)]
#[path = "dart_http_tests.rs"]
mod tests;
