use regex::Regex;
use std::sync::OnceLock;

pub(super) fn extract_routes(source: &str) -> Vec<(String, String)> {
    let mut routes = extract_to_routes(source);
    routes.extend(expand_resources(source));
    routes.sort();
    routes.dedup();
    routes
}

fn extract_to_routes(source: &str) -> Vec<(String, String)> {
    rails_route_re()
        .captures_iter(source)
        .filter_map(|cap| {
            Some((
                cap.get(1)?.as_str().to_string(),
                cap.get(2)?.as_str().to_string(),
            ))
        })
        .collect()
}

fn expand_resources(source: &str) -> Vec<(String, String)> {
    let mut namespace_depth: usize = 0;
    let mut routes = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim_start();
        if namespace_do_re().is_match(trimmed) {
            namespace_depth += 1;
        }
        if namespace_depth == 0 {
            if let Some(cap) = rails_resources_re().captures(line) {
                let name = cap.get(1).or_else(|| cap.get(2)).unwrap();
                routes.extend(resource_rest_routes(name.as_str()));
            }
        }
        if end_re().is_match(trimmed) {
            namespace_depth = namespace_depth.saturating_sub(1);
        }
    }
    routes
}

fn resource_rest_routes(name: &str) -> Vec<(String, String)> {
    let collection = format!("/{name}");
    let member = format!("/{name}/:id");
    vec![
        (collection.clone(), format!("{name}#index")),
        (member.clone(), format!("{name}#show")),
        (collection, format!("{name}#create")),
        (member.clone(), format!("{name}#update")),
        (member, format!("{name}#destroy")),
    ]
}

fn rails_route_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"(?m)^\s*(?:get|post|put|patch|delete)\s+["']([^"']+)["']\s*,\s*to:\s*["']([^"']+)["']"#,
        )
        .expect("route")
    })
}

fn rails_resources_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    // 0–2 spaces so deeply indented nested resources stay a non-edge.
    RE.get_or_init(|| {
        Regex::new(r#"(?m)^[ \t]{0,2}resources\s+(?::([a-z]\w*)|["']([a-z]\w*)["'])\s*$"#)
            .expect("resources")
    })
}

fn namespace_do_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"^namespace\s+(?::[a-z]\w*|["'][a-z]\w*["'])\s+do\b"#).expect("namespace")
    })
}

fn end_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^end\b").expect("end"))
}

#[cfg(test)]
#[path = "ruby_http_tests.rs"]
mod tests;
