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
    if raw.starts_with("//") || raw.contains(['$', '{', '%']) {
        return None;
    }
    if raw.starts_with('/') {
        return Some(normalize_route_pattern(raw));
    }
    let url = url::Url::parse(raw).ok()?;
    if !matches!(url.scheme(), "http" | "https") || !loopback_host(url.host()) {
        return None;
    }
    Some(normalize_route_pattern(url.path()))
}

fn loopback_host(host: Option<url::Host<&str>>) -> bool {
    match host {
        Some(url::Host::Domain("localhost")) => true,
        Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
        Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
        _ => false,
    }
}

pub(crate) fn normalize_route_pattern(raw: &str) -> String {
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
