use super::comments::strip_comments;
pub(super) mod scanner;
pub(super) use scanner::matching_brace;
use scanner::{assignment_index, matching_delimiter, top_level_value_end};

use regex::Regex;
use std::collections::BTreeSet;

pub(super) fn const_object_body(source: &str, target: &str) -> Option<String> {
    let source = strip_comments(source);
    let pattern = format!(r#"\bconst\s+{}\b"#, regex::escape(target));
    let start = Regex::new(&pattern)
        .ok()
        .and_then(|regex| const_object_start(&source, &regex))?;
    let assignment = assignment_index(&source, start)?;
    let open = source[assignment..].find('{')? + assignment;
    let close = matching_brace(&source, open)?;
    source.get(open + 1..close).map(str::to_string)
}

pub(super) fn const_array_body(source: &str, target: &str) -> Option<String> {
    let source = strip_comments(source);
    let pattern = format!(r#"\bconst\s+{}\b"#, regex::escape(target));
    let start = Regex::new(&pattern)
        .ok()
        .and_then(|regex| const_object_start(&source, &regex))?;
    let assignment = assignment_index(&source, start)?;
    let initializer = source[assignment..].trim_start();
    if !initializer.starts_with('[') {
        return None;
    }
    let open = assignment + source[assignment..].len() - initializer.len();
    let close = matching_delimiter(&source, open, '[', ']')?;
    source.get(open + 1..close).map(str::to_string)
}

fn const_object_start(source: &str, regex: &Regex) -> Option<usize> {
    let mut quote = None;
    let mut escaped = false;
    for (idx, ch) in source.char_indices() {
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == active_quote {
                quote = None;
            }
            continue;
        }
        if ch == '"' || ch == '\'' || ch == '`' {
            quote = Some(ch);
            continue;
        }
        if let Some(mat) = regex.find(&source[idx..]).filter(|mat| mat.start() == 0) {
            return Some(idx + mat.end());
        }
    }
    None
}

pub(super) fn top_level_object_keys(body: &str) -> BTreeSet<String> {
    top_level_entries(body)
        .into_iter()
        .map(|(key, _)| key)
        .collect()
}

pub(super) fn top_level_property_values(body: &str, property: &str) -> BTreeSet<String> {
    top_level_entries(body)
        .into_iter()
        .filter_map(|(_, value)| direct_object_body(&value).map(|body| top_level_entries(&body)))
        .flatten()
        .filter_map(|(key, value)| {
            (key == property)
                .then(|| quoted_string_literal(value.trim()))
                .flatten()
        })
        .collect()
}

pub(super) fn top_level_entries(body: &str) -> Vec<(String, String)> {
    let mut entries = Vec::new();
    let mut rest = body;
    while let Some((key, after_key)) = take_key(rest) {
        let Some(colon) = after_key.find(':') else {
            break;
        };
        let value = &after_key[colon + 1..];
        let end = top_level_value_end(value);
        entries.push((key, value[..end].to_string()));
        rest = trim_ignorable(&value[end..]);
    }
    entries
}

fn take_key(source: &str) -> Option<(String, &str)> {
    let source = trim_ignorable(source);
    let mut chars = source.char_indices();
    let (_, first) = chars.next()?;
    if first == '"' || first == '\'' {
        let (key, rest) = take_quoted_key(source, first)?;
        return Some((key, rest));
    }
    let end = source
        .char_indices()
        .find(|(_, ch)| !(ch.is_alphanumeric() || *ch == '_' || *ch == '$' || *ch == '-'))
        .map(|(idx, _)| idx)?;
    Some((source[..end].to_string(), &source[end..]))
}

fn take_quoted_key(source: &str, quote: char) -> Option<(String, &str)> {
    let mut key = String::new();
    let mut escaped = false;
    for (idx, ch) in source[1..].char_indices() {
        if escaped {
            key.push(ch);
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == quote {
            return Some((key, &source[idx + 2..]));
        }
        key.push(ch);
    }
    None
}

fn trim_ignorable(mut source: &str) -> &str {
    loop {
        let trimmed = source.trim_start_matches(|ch: char| ch == ',' || ch.is_whitespace());
        if let Some(rest) = trimmed.strip_prefix("...") {
            let end = top_level_value_end(rest);
            if end == rest.len() {
                return "";
            }
            source = &rest[end + 1..];
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("__comment__") {
            source = rest;
            continue;
        }
        if trimmed.starts_with('[') {
            let end = top_level_value_end(trimmed);
            if end == trimmed.len() {
                return "";
            }
            source = &trimmed[end + 1..];
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("//") {
            source = rest.split_once('\n').map_or("", |(_, after)| after);
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("/*") {
            let Some((_, after)) = rest.split_once("*/") else {
                return "";
            };
            source = after;
            continue;
        }
        return trimmed;
    }
}

pub(super) fn direct_object_body(value: &str) -> Option<String> {
    let value = value.trim();
    if !value.starts_with('{') {
        return None;
    }
    let close = matching_brace(value, 0)?;
    value.get(1..close).map(str::to_string)
}

pub(super) fn quoted_string_literal(value: &str) -> Option<String> {
    let quote = value.chars().next()?;
    if quote != '"' && quote != '\'' {
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
