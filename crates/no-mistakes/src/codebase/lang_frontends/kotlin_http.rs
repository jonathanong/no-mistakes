use regex::Regex;
use std::sync::OnceLock;

pub(super) fn extract_http_routes(source: &str) -> Vec<(String, String)> {
    let prefix = class_request_mapping(source);
    let mut routes = extract_method_mappings(source);
    for (path, _) in &mut routes {
        *path = join_route(prefix.as_deref(), path);
    }
    routes.sort();
    routes.dedup();
    routes
}

fn class_request_mapping(source: &str) -> Option<String> {
    let class_at = kotlin_class_re().find(source)?.start();
    request_mapping_re()
        .captures_iter(&source[..class_at])
        .last()
        .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
}

fn extract_method_mappings(source: &str) -> Vec<(String, String)> {
    method_mapping_re()
        .captures_iter(source)
        .filter_map(|cap| {
            Some((
                cap.get(1)?.as_str().to_string(),
                cap.get(2)?.as_str().to_string(),
            ))
        })
        .collect()
}

fn join_route(prefix: Option<&str>, path: &str) -> String {
    let path = normalize_path(path);
    match prefix.map(str::trim).filter(|value| !value.is_empty()) {
        None => path,
        Some(prefix) => format!("{}{path}", normalize_path(prefix).trim_end_matches('/')),
    }
}

fn normalize_path(raw: &str) -> String {
    if raw.starts_with('/') {
        raw.to_string()
    } else {
        format!("/{raw}")
    }
}

fn kotlin_class_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b(?:class|interface|object)\s+[A-Za-z_]").expect("class"))
}

fn request_mapping_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"@RequestMapping\(\s*(?:(?:path|value)\s*=\s*)?"([^"]+)"\s*\)"#)
            .expect("request mapping")
    })
}

fn method_mapping_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"(?s)@(?:(?:Get|Post|Put|Patch|Delete)Mapping|RequestMapping)\(\s*(?:(?:path|value)\s*=\s*)?"([^"]+)"\s*\)(?:\s*@[A-Za-z_.]+(?:\([^)]*\))?)*\s*(?:(?:public|private|protected|internal|open|override|suspend|inline|tailrec|actual|expect)\s+)*fun\s+([A-Za-z_][A-Za-z0-9_]*)\s*\("#,
        )
        .expect("method mapping")
    })
}

#[cfg(test)]
#[path = "kotlin_http_tests.rs"]
mod tests;
