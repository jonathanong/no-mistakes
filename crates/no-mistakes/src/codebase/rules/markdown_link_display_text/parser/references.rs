use std::collections::HashMap;

use super::InlineLink;

pub(super) fn definitions(source: &str) -> HashMap<String, String> {
    let mut definitions = HashMap::new();
    for line in source.split_inclusive('\n') {
        if let Some((label, href)) = parse_definition(line) {
            definitions.entry(label).or_insert(href);
        }
    }
    definitions
}

pub(super) fn parse_link(
    source: &str,
    start: usize,
    reference_definitions: &HashMap<String, String>,
) -> Option<(InlineLink, usize)> {
    let bytes = source.as_bytes();
    let text_end = super::find_byte(bytes, start + 1, b']')?;
    if bytes.get(text_end + 1) != Some(&b'[') {
        return None;
    }
    let label_start = text_end + 2;
    let label_end = super::find_byte(bytes, label_start, b']')?;
    let label = if label_start == label_end {
        normalize_label(&source[start + 1..text_end])
    } else {
        normalize_label(source[label_start..label_end].trim())
    };
    if label.is_empty() {
        return None;
    }
    let href = reference_definitions.get(&label)?.clone();
    Some((
        InlineLink {
            text: source[start + 1..text_end].to_string(),
            href,
            offset: start,
        },
        label_end + 1,
    ))
}

fn parse_definition(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim_start_matches([' ', '\t']);
    let indent = line.len() - trimmed.len();
    if indent > 3 || !trimmed.starts_with('[') {
        return None;
    }
    let label_end = super::find_byte(trimmed.as_bytes(), 1, b']')?;
    let mut rest = &trimmed[label_end + 1..];
    rest = rest.trim_start_matches([' ', '\t']);
    if !rest.starts_with(':') {
        return None;
    }
    let label = normalize_label(&trimmed[1..label_end]);
    if label.is_empty() {
        return None;
    }
    let href = href_destination(rest[1..].trim());
    (!href.is_empty()).then_some((label, href.to_string()))
}

fn href_destination(value: &str) -> &str {
    if let Some(rest) = value.strip_prefix('<') {
        if let Some(end) = rest.find('>') {
            &rest[..end]
        } else {
            value
        }
    } else {
        value.split_ascii_whitespace().next().unwrap_or_default()
    }
}

fn normalize_label(label: &str) -> String {
    label
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}
