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
    let mut routes = Vec::new();
    for cap in rails_resources_re().captures_iter(source) {
        let Some(name) = cap.get(1).or_else(|| cap.get(2)).map(|m| m.as_str()) else {
            continue;
        };
        routes.extend(resource_rest_routes(name));
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
    // 0–2 spaces so namespaced `    resources :users` stays a non-edge.
    RE.get_or_init(|| {
        Regex::new(r#"(?m)^[ \t]{0,2}resources\s+(?::([a-z]\w*)|["']([a-z]\w*)["'])\s*$"#)
            .expect("resources")
    })
}

#[cfg(test)]
#[path = "ruby_http_tests.rs"]
mod tests;
