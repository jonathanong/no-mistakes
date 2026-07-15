use super::scanner::{
    find_label_colon, find_matching_delimiter, read_quoted_string, string_arg, Scanner,
};
use super::SwiftTargetFacts;

pub(super) fn parse_manifest_targets(source: &str) -> Vec<SwiftTargetFacts> {
    target_calls(source)
        .into_iter()
        .filter_map(|call| {
            let name = string_arg(call.body, "name")?;
            let dependencies = dependencies_body(call.body)
                .map(manifest_dependency_names)
                .unwrap_or_default();
            Some(SwiftTargetFacts {
                name,
                is_test: call.is_test,
                dependencies,
                roots: Vec::new(),
            })
        })
        .collect()
}

struct TargetCall<'a> {
    is_test: bool,
    body: &'a str,
}

fn target_calls(source: &str) -> Vec<TargetCall<'_>> {
    let mut calls = Vec::new();
    let mut scanner = Scanner::new(source);
    while let Some(index) = scanner.next_code_index() {
        let rest = &source[index..];
        let (is_test, open_index) = if rest.starts_with(".testTarget(") {
            (true, index + ".testTarget".len())
        } else if rest.starts_with(".target(") {
            (false, index + ".target".len())
        } else {
            continue;
        };
        if let Some(close_index) = find_matching_delimiter(source, open_index, '(', ')') {
            calls.push(TargetCall {
                is_test,
                body: &source[open_index + 1..close_index],
            });
            scanner.skip_to(close_index + 1);
        }
    }
    calls
}

fn dependencies_body(target_body: &str) -> Option<&str> {
    let label = find_label_colon(target_body, "dependencies")?;
    let open_bracket = target_body[label + 1..]
        .char_indices()
        .find_map(|(offset, ch)| (!ch.is_whitespace()).then_some((label + 1 + offset, ch)))?;
    if open_bracket.1 != '[' {
        return None;
    }
    let close_bracket = find_matching_delimiter(target_body, open_bracket.0, '[', ']')?;
    target_body.get(open_bracket.0 + 1..close_bracket)
}

fn manifest_dependency_names(dependencies_body: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut scanner = Scanner::new(dependencies_body);
    while let Some(index) = scanner.next_code_index() {
        let rest = &dependencies_body[index..];
        if rest.starts_with('"') {
            if let Some((value, next)) = read_quoted_string(dependencies_body, index) {
                names.push(value);
                scanner.skip_to(next);
            }
            continue;
        }
        let open = if rest.starts_with(".target(") {
            Some(index + ".target".len())
        } else if rest.starts_with(".product(") {
            Some(index + ".product".len())
        } else if rest.starts_with(".byName(") {
            Some(index + ".byName".len())
        } else {
            None
        };
        if let Some(open) = open {
            if let Some(close) = find_matching_delimiter(dependencies_body, open, '(', ')') {
                if let Some(name) = string_arg(&dependencies_body[open + 1..close], "name") {
                    names.push(name);
                }
                scanner.skip_to(close + 1);
            }
        }
    }
    names
}

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
