use regex::Regex;
use std::sync::OnceLock;

pub(super) fn extract_http_routes(source: &str) -> Vec<(String, String)> {
    let mut routes = extract_pairs(source, axum_route_re(), handler_from_path);
    routes.extend(extract_actix_resources(source));
    routes.extend(extract_attr_routes(source));
    routes.sort();
    routes.dedup();
    routes
}

fn extract_pairs(source: &str, re: &Regex, handler: fn(&str) -> String) -> Vec<(String, String)> {
    re.captures_iter(source)
        .filter_map(|cap| {
            Some((
                cap.get(1)?.as_str().to_string(),
                handler(cap.get(2)?.as_str()),
            ))
        })
        .collect()
}

fn handler_from_path(raw: &str) -> String {
    raw.rsplit("::").next().unwrap_or(raw).to_string()
}

fn extract_actix_resources(source: &str) -> Vec<(String, String)> {
    let mut routes = Vec::new();
    for cap in actix_resource_start_re().captures_iter(source) {
        let Some(path) = cap.get(1) else { continue };
        let Some(matched) = cap.get(0) else { continue };
        let rest = source.get(matched.end()..).unwrap_or("");
        let stmt = rest.split(';').next().unwrap_or(rest);
        for route in actix_route_re().captures_iter(stmt) {
            let Some(handler) = route.get(1) else {
                continue;
            };
            routes.push((
                path.as_str().to_string(),
                handler_from_path(handler.as_str()),
            ));
        }
    }
    routes
}

fn extract_attr_routes(source: &str) -> Vec<(String, String)> {
    attr_route_re()
        .captures_iter(source)
        .filter_map(|cap| {
            Some((
                cap.get(1)?.as_str().to_string(),
                cap.get(2)?.as_str().to_string(),
            ))
        })
        .collect()
}

fn axum_route_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"\.route\(\s*"(/(?:[^"]*))"\s*,\s*(?:get|post|put|patch|delete|head|options|trace|any)\(\s*([A-Za-z_][\w:]*)\s*\)\s*\)"#,
        )
        .expect("axum")
    })
}

fn actix_resource_start_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"web::resource\(\s*"(/(?:[^"]*))"\s*\)"#).expect("actix start"))
}

fn actix_route_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"\.route\(\s*web::(?:get|post|put|patch|delete|options)\(\s*\)\s*\.to\(\s*([A-Za-z_][\w:]*)\s*\)"#,
        )
        .expect("actix route")
    })
}

fn attr_route_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"(?s)#\[(?:[A-Za-z_]\w*::)*(?:get|post|put|patch|delete|head|options)\(\s*"(/(?:[^"]*))"[^)]*\)\]\s*(?:pub(?:\([^)]+\))?\s+)?(?:async\s+)?fn\s+([A-Za-z_]\w*)"#,
        )
        .expect("attr route")
    })
}

#[cfg(test)]
#[path = "rust_http_tests.rs"]
mod tests;
