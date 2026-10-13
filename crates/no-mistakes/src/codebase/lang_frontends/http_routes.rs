use regex::Regex;
use std::sync::OnceLock;

pub(super) fn class_request_mapping(source: &str, class: &Regex) -> Option<String> {
    let class_at = class.find(source)?.start();
    request_mapping_re()
        .captures_iter(&source[..class_at])
        .last()
        .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
}

pub(super) fn extract_method_mappings(source: &str, method: &Regex) -> Vec<(String, String)> {
    method
        .captures_iter(source)
        .filter_map(|cap| {
            Some((
                cap.get(1)?.as_str().to_string(),
                cap.get(2)?.as_str().to_string(),
            ))
        })
        .collect()
}

pub(super) fn join_route(prefix: Option<&str>, path: &str) -> String {
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

fn request_mapping_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"@RequestMapping\(\s*(?:(?:path|value)\s*=\s*)?"([^"]+)"\s*\)"#)
            .expect("request mapping")
    })
}
