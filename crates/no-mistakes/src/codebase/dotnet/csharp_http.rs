use regex::Regex;
use std::sync::OnceLock;

pub(crate) fn extract_http_routes(source: &str) -> Vec<(String, String)> {
    let mut routes = extract_minimal_apis(source);
    routes.extend(extract_http_attributes(source));
    routes.sort();
    routes.dedup();
    routes
}

fn extract_minimal_apis(source: &str) -> Vec<(String, String)> {
    minimal_api_re()
        .captures_iter(source)
        .filter_map(|cap| {
            Some((
                normalize_route_path(cap.get(1)?.as_str()),
                cap.get(2)?.as_str().to_string(),
            ))
        })
        .collect()
}

fn extract_http_attributes(source: &str) -> Vec<(String, String)> {
    http_attr_re()
        .captures_iter(source)
        .filter_map(|cap| {
            Some((
                normalize_route_path(cap.get(1)?.as_str()),
                cap.get(2)?.as_str().to_string(),
            ))
        })
        .collect()
}

fn normalize_route_path(raw: &str) -> String {
    if raw.starts_with('/') {
        raw.to_string()
    } else {
        format!("/{raw}")
    }
}

fn minimal_api_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"\.Map(?:Get|Post|Put|Patch|Delete)\(\s*"([^"]+)"\s*,\s*([A-Za-z_][A-Za-z0-9_.]*)"#,
        )
        .expect("aspnet map")
    })
}

fn http_attr_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"(?s)\[(?:(?:[A-Za-z_]\w*\.)+)?Http(?:Get|Post|Put|Patch|Delete)\(\s*"([^"]+)"\s*\)\]\s*(?:(?:public|private|internal|protected|static|async|virtual|override|new)\s+)*(?:[\w.<>,\[\]?]+\s+)+([A-Za-z_][A-Za-z0-9_]*)\s*\("#,
        )
        .expect("aspnet attr")
    })
}

#[cfg(test)]
#[path = "csharp_http_tests.rs"]
mod tests;
