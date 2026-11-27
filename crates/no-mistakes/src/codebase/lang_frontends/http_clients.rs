use regex::Regex;

#[path = "http_client_regex.rs"]
mod regexes;

pub(crate) fn extract_python(source: &str) -> Vec<String> {
    paths_from(source, &[regexes::python()])
}

pub(crate) fn extract_go(source: &str) -> Vec<String> {
    paths_from(
        source,
        &[
            regexes::go_get(),
            regexes::go_request(),
            regexes::go_request_ctx(),
        ],
    )
}

pub(crate) fn extract_rust(source: &str) -> Vec<String> {
    paths_from(source, &[regexes::rust_get(), regexes::rust_client()])
}

pub(crate) fn extract_ruby(source: &str) -> Vec<String> {
    paths_from(source, &[regexes::ruby(), regexes::ruby_net()])
}

pub(crate) fn extract_php(source: &str) -> Vec<String> {
    paths_from(source, &[regexes::php(), regexes::php_request()])
}

pub(crate) fn extract_jvm(source: &str) -> Vec<String> {
    paths_from(source, &[regexes::jvm()])
}

pub(crate) fn extract_elixir(source: &str) -> Vec<String> {
    paths_from(source, &[regexes::elixir()])
}

pub(crate) fn extract_dotnet(source: &str) -> Vec<String> {
    paths_from(source, &[regexes::dotnet(), regexes::dotnet_message()])
}

pub(crate) fn client_matches_route(client: &str, route: &str) -> bool {
    crate::codebase::ts_routes::matcher::matches(client, &normalize_route_pattern(route))
}

fn paths_from(source: &str, patterns: &[&Regex]) -> Vec<String> {
    let mut paths = Vec::new();
    for pattern in patterns {
        paths.extend(
            pattern
                .captures_iter(source)
                .filter_map(|cap| static_path(first_group(&cap))),
        );
    }
    paths.sort();
    paths.dedup();
    paths
}

fn first_group<'a>(cap: &'a regex::Captures<'a>) -> Option<&'a str> {
    cap.iter()
        .skip(1)
        .flatten()
        .map(|group| group.as_str())
        .next()
}

fn static_path(raw: Option<&str>) -> Option<String> {
    let raw = raw?;
    if raw.contains(['$', '{', '%']) {
        return None;
    }
    let path = hosted_path(raw).unwrap_or(raw);
    let path = path.split(['?', '#']).next().unwrap_or(path);
    let path = if path.len() > 1 {
        path.trim_end_matches('/')
    } else {
        path
    };
    path.starts_with('/').then(|| path.to_string())
}

fn hosted_path(raw: &str) -> Option<&str> {
    let rest = raw.split_once("://")?.1;
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    rest[end..].starts_with('/').then_some(&rest[end..])
}

fn normalize_route_pattern(raw: &str) -> String {
    let path = raw.split(['?', '#']).next().unwrap_or(raw);
    let path = if path.len() > 1 {
        path.trim_end_matches('/')
    } else {
        path
    };
    if path.is_empty() {
        return "/".to_string();
    }
    if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
#[path = "http_clients_tests.rs"]
mod tests;
