use super::object::scanner::top_level_value_end;
use super::object::{const_array_body, direct_object_body, top_level_entries};
use std::collections::BTreeSet;

pub(super) fn extract_ts_array_literal(source: &str, target: &str) -> BTreeSet<String> {
    let Some(body) = const_array_body(source, target) else {
        return BTreeSet::new();
    };
    top_level_values(&body)
        .into_iter()
        .filter_map(|value| quoted_string_literal(value.trim()))
        .collect()
}

pub(super) fn extract_ts_const_array_property(
    source: &str,
    target: &str,
    property: &str,
) -> BTreeSet<String> {
    let Some(body) = const_array_body(source, target) else {
        return BTreeSet::new();
    };
    top_level_values(&body)
        .into_iter()
        .filter_map(|value| direct_object_body(&value).map(|body| top_level_entries(&body)))
        .flatten()
        .filter_map(|(key, value)| {
            (key == property)
                .then(|| quoted_string_literal(value.trim()))
                .flatten()
        })
        .collect()
}

pub(super) fn top_level_values(body: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut rest = trim_ignorable(body);
    while !rest.is_empty() {
        let end = top_level_value_end(rest);
        values.push(rest[..end].trim().to_string());
        rest = trim_ignorable(&rest[end..]);
    }
    values
}

fn trim_ignorable(source: &str) -> &str {
    let mut rest = source;
    loop {
        rest = rest.trim_start_matches(|ch: char| ch == ',' || ch.is_whitespace());
        if let Some(after_comment) = rest.strip_prefix("//") {
            rest = after_comment
                .find('\n')
                .map(|index| &after_comment[index + 1..])
                .unwrap_or("");
            continue;
        }
        if let Some(after_comment) = rest.strip_prefix("/*") {
            rest = after_comment
                .find("*/")
                .map(|index| &after_comment[index + 2..])
                .unwrap_or("");
            continue;
        }
        return rest;
    }
}

pub(super) fn quoted_string_literal(value: &str) -> Option<String> {
    let quote = value.chars().next()?;
    if quote != '"' && quote != '\'' && quote != '`' {
        return None;
    }
    let mut literal = String::new();
    let mut escaped = false;
    for ch in value[1..].chars() {
        if escaped {
            literal.push(ch);
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == quote {
            return Some(literal);
        }
        literal.push(ch);
    }
    None
}
