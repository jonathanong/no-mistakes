use regex::Regex;

#[path = "http_client_regex.rs"]
mod regexes;

pub(crate) fn extract_python(source: &str, masked: &str) -> Vec<String> {
    let mut positions = source.char_indices().zip(masked.chars());
    paths_from_matching(source, &[regexes::python()], |start| {
        positions
            .find(|((offset, _), _)| *offset == start)
            .is_some_and(|(_, code)| code != ' ')
    })
}

pub(crate) fn extract_go(source: &str) -> Vec<String> {
    let mut paths = paths_from(source, &[regexes::go_get(), regexes::go_request()]);
    paths.extend(
        regexes::go_request_with_context()
            .find_iter(source)
            .filter_map(|call| {
                let arguments = after_context_argument(&source[call.end()..])?;
                let method_and_url = regexes::go_method_and_url().captures(arguments)?;
                static_path(first_group(&method_and_url))
            }),
    );
    paths.sort();
    paths.dedup();
    paths
}

fn after_context_argument(arguments: &str) -> Option<&str> {
    let mut depth = 0;
    for token in regexes::go_context_tokens().find_iter(arguments) {
        match token.as_str() {
            "(" | "[" | "{" => depth += 1,
            ")" | "]" | "}" if depth == 0 => return None,
            ")" | "]" | "}" => depth -= 1,
            "," if depth == 0 => return arguments.get(token.end()..),
            "'" | "\"" | "`" => return None,
            _ => {}
        }
    }
    None
}

pub(crate) fn extract_rust(source: &str) -> Vec<String> {
    paths_from(source, &[regexes::rust_get(), regexes::rust_client()])
}

pub(crate) fn extract_ruby(source: &str) -> Vec<String> {
    paths_from(
        source,
        &[regexes::ruby(), regexes::ruby_bare(), regexes::ruby_net()],
    )
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
    paths_from_matching(source, patterns, |_| true)
}

fn paths_from_matching(
    source: &str,
    patterns: &[&Regex],
    mut in_code: impl FnMut(usize) -> bool,
) -> Vec<String> {
    let mut paths = Vec::new();
    for pattern in patterns {
        paths.extend(pattern.captures_iter(source).filter_map(|cap| {
            in_code(cap.get(0)?.start())
                .then(|| static_path(first_group(&cap)))
                .flatten()
        }));
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
    if raw.starts_with("//") || raw.contains(['$', '{']) || !valid_percent_encoding(raw) {
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

fn valid_percent_encoding(path: &str) -> bool {
    path.as_bytes()
        .iter()
        .enumerate()
        .filter(|(_, byte)| **byte == b'%')
        .all(|(index, _)| {
            path.as_bytes()
                .get(index + 1..index + 3)
                .is_some_and(|pair| pair.iter().all(u8::is_ascii_hexdigit))
        })
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
    let mut normalized = String::new();
    for segment in path.trim_start_matches('/').split('/') {
        normalized.push('/');
        if let Some(parameter) = segment
            .strip_prefix('{')
            .and_then(|part| part.strip_suffix('}'))
        {
            let name = parameter.split(':').next().unwrap_or(parameter);
            normalized.push(':');
            normalized.push_str(name);
        } else {
            normalized.push_str(segment);
        }
    }
    normalized
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
#[path = "http_clients_tests.rs"]
mod tests;
