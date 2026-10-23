use regex::Regex;
use std::sync::OnceLock;

pub(super) fn extract_http_routes(source: &str) -> Vec<(String, String)> {
    let source = super::super::strip::mask_triple_quoted_strings(source);
    let mut prefixes: Vec<(usize, Option<String>)> = Vec::new();
    let mut routes = Vec::new();
    let mut opaque_rest = false;
    for line in source.lines() {
        let indent = line.len() - line.trim_start().len();
        let trimmed = line.trim();
        if opaque_rest || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if is_end_line(trimmed) {
            while prefixes.last().is_some_and(|(open, _)| *open >= indent) {
                prefixes.pop();
            }
            continue;
        }
        let scope = parse_scope(trimmed);
        let verb = scope.is_none().then(|| parse_verb(trimmed)).flatten();
        if (scope.is_some() || verb.is_some())
            && prefixes.last().is_some_and(|(open, _)| indent <= *open)
        {
            opaque_rest = true;
            continue;
        }
        if let Some((prefix, inline)) = scope {
            if inline {
                if !prefixes.iter().any(|(_, open)| open.is_none()) {
                    if let (Some(prefix), Some((path, handler))) = (
                        prefix.as_deref(),
                        inline_scope_verb(trimmed).and_then(parse_verb),
                    ) {
                        routes.push((join_all(&prefixes, Some(prefix), &path), handler));
                    }
                }
                continue;
            }
            prefixes.push((indent, prefix));
            continue;
        }
        if prefixes.iter().any(|(_, prefix)| prefix.is_none()) {
            continue;
        }
        if let Some((path, handler)) = verb {
            routes.push((join_all(&prefixes, None, &path), handler));
        }
    }
    routes.sort();
    routes.dedup();
    routes
}

fn join_all(prefixes: &[(usize, Option<String>)], extra: Option<&str>, path: &str) -> String {
    let prefix = prefixes
        .iter()
        .filter_map(|(_, prefix)| prefix.as_deref())
        .chain(extra)
        .fold(None, |acc, part| {
            Some(super::super::http_routes::join_route(acc.as_deref(), part))
        });
    super::super::http_routes::join_route(prefix.as_deref(), path)
}

fn is_end_line(trimmed: &str) -> bool {
    let Some(rest) = trimmed.strip_prefix("end") else {
        return false;
    };
    rest.is_empty()
        || rest
            .chars()
            .next()
            .is_some_and(|ch| !ch.is_ascii_alphanumeric() && ch != '_')
}

fn parse_scope(trimmed: &str) -> Option<(Option<String>, bool)> {
    if !(trimmed.starts_with("scope ") || trimmed.starts_with("scope(")) {
        return None;
    }
    let prefix = phoenix_scope_re().captures(trimmed).and_then(|cap| {
        let path = cap.get(1)?.as_str();
        (!path.contains("#{")).then(|| path.to_string())
    });
    Some((prefix, is_inline_scope(trimmed)))
}

fn is_inline_scope(trimmed: &str) -> bool {
    let code = code_without_comment(trimmed);
    code.contains(", do:")
        || (code.split_whitespace().last() == Some("end")
            && code.split_whitespace().any(|word| word == "do"))
}

fn inline_scope_verb(trimmed: &str) -> Option<&str> {
    let code = code_without_comment(trimmed);
    let after = code.split_once(" do ")?.1.trim();
    Some(after.strip_suffix("end")?.trim())
}

fn code_without_comment(trimmed: &str) -> &str {
    let mut in_string = false;
    let mut quote = '\0';
    for (i, ch) in trimmed.char_indices() {
        if in_string {
            if ch == quote {
                in_string = false;
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            in_string = true;
            quote = ch;
        } else if ch == '#' {
            return trimmed[..i].trim_end();
        }
    }
    trimmed
}

fn parse_verb(trimmed: &str) -> Option<(String, String)> {
    let cap = phoenix_route_re().captures(trimmed)?;
    Some((
        cap.get(1)?.as_str().to_string(),
        format!("{}.{}", cap.get(2)?.as_str(), cap.get(3)?.as_str()),
    ))
}

fn phoenix_scope_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"^scope\s+["']([^"']+)["']\s*(?:,|\s+do\b|$)"#).expect("phoenix scope")
    })
}

fn phoenix_route_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"^(?:get|post|put|patch|delete)\s+["']([^"']+)["']\s*,\s*([A-Z][A-Za-z0-9_.]*)\s*,\s*:([a-z_][A-Za-z0-9_]*)"#,
        )
        .expect("phoenix route")
    })
}

#[cfg(test)]
#[path = "elixir_http_tests.rs"]
mod tests;
