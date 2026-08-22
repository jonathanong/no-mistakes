use regex::Regex;
use std::sync::OnceLock;

pub(super) fn extract_http_routes(source: &str) -> Vec<(String, String)> {
    let mut routes = extract_pairs(source, net_http_route_re());
    routes.extend(extract_pairs(source, mux_route_re()));
    routes.sort();
    routes.dedup();
    routes
}

fn extract_pairs(source: &str, re: &Regex) -> Vec<(String, String)> {
    re.captures_iter(source)
        .filter_map(|cap| {
            let path = cap.get(1).or(cap.get(2))?.as_str();
            Some((path.to_string(), cap.get(3)?.as_str().to_string()))
        })
        .collect()
}

fn net_http_route_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"\bHandle(?:Func)?\(\s*(?:"(/[^"]*)"|`(/[^`]*)`)\s*,\s*([A-Za-z_][\w.]*)"#)
            .expect("net/http")
    })
}

fn mux_route_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"\.(?:Get|Post|Put|Patch|Delete|Head|Options|Connect|Trace|GET|POST|PUT|PATCH|DELETE|HEAD|OPTIONS)\(\s*(?:"(/[^"]*)"|`(/[^`]*)`)\s*,\s*([A-Za-z_][\w.]*)"#,
        )
        .expect("go mux")
    })
}

#[cfg(test)]
#[path = "go_http_tests.rs"]
mod tests;
