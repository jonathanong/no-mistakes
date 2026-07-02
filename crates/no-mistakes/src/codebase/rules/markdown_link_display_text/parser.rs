use std::collections::HashMap;

mod references;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct InlineLink {
    pub(super) text: String,
    pub(super) href: String,
    pub(super) offset: usize,
}

pub(super) fn markdown_links_outside_code(source: &str) -> Vec<InlineLink> {
    let fenced = strip_fenced_code(source);
    let reference_definitions = references::definitions(&fenced);
    scan_links(&fenced, &reference_definitions)
}

fn scan_links(source: &str, reference_definitions: &HashMap<String, String>) -> Vec<InlineLink> {
    let bytes = source.as_bytes();
    let mut links = Vec::new();
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'\\' {
            index = (index + 2).min(bytes.len());
        } else if bytes[index] == b'`' {
            index = skip_inline_code(bytes, index);
        } else if bytes[index] == b'[' && (index == 0 || bytes[index - 1] != b'!') {
            if let Some((link, next)) = parse_inline_link(source, index) {
                links.push(link);
                index = next;
            } else if let Some((link, next)) =
                references::parse_link(source, index, reference_definitions)
            {
                links.push(link);
                index = next;
            } else {
                index += 1;
            }
        } else {
            index += 1;
        }
    }
    links
}

fn skip_inline_code(bytes: &[u8], start: usize) -> usize {
    let marker_len = count_backticks(bytes, start);
    let mut index = start + marker_len;
    while index < bytes.len() {
        if bytes[index] == b'\\' {
            index = (index + 2).min(bytes.len());
        } else if bytes[index] == b'`' {
            let close_len = count_backticks(bytes, index);
            if close_len == marker_len {
                return index + close_len;
            }
            index += close_len;
        } else {
            index += 1;
        }
    }
    start + marker_len
}

fn count_backticks(bytes: &[u8], start: usize) -> usize {
    bytes[start..]
        .iter()
        .take_while(|byte| **byte == b'`')
        .count()
}

pub(super) fn parse_inline_link(source: &str, start: usize) -> Option<(InlineLink, usize)> {
    let bytes = source.as_bytes();
    let text_end = find_byte(bytes, start + 1, b']')?;
    if bytes.get(text_end + 1) != Some(&b'(') {
        return None;
    }
    let href_start = text_end + 2;
    let href_end = find_link_destination_end(bytes, href_start)?;
    Some((
        InlineLink {
            text: source[start + 1..text_end].to_string(),
            href: source[href_start..href_end].to_string(),
            offset: start,
        },
        href_end + 1,
    ))
}

fn find_link_destination_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut index = start;
    let mut paren_depth = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index = (index + 2).min(bytes.len()),
            b'(' => {
                paren_depth += 1;
                index += 1;
            }
            b')' if paren_depth == 0 => return Some(index),
            b')' => {
                paren_depth -= 1;
                index += 1;
            }
            _ => index += 1,
        }
    }
    None
}

fn find_byte(bytes: &[u8], start: usize, target: u8) -> Option<usize> {
    bytes
        .iter()
        .enumerate()
        .skip(start)
        .find_map(|(index, byte)| (*byte == target).then_some(index))
}

pub(super) fn strip_fenced_code(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut in_fence: Option<Fence> = None;
    for line in source.split_inclusive('\n') {
        let marker = fence_marker(line, in_fence.is_none());
        match (in_fence, marker) {
            (None, Some(marker)) => {
                in_fence = Some(marker);
                push_masked_line(line, &mut out);
            }
            (Some(active), Some(marker))
                if active.marker == marker.marker && marker.len >= active.len =>
            {
                in_fence = None;
                push_masked_line(line, &mut out);
            }
            (Some(_), _) => {
                push_masked_line(line, &mut out);
            }
            (None, _) => out.push_str(line),
        }
    }
    out
}

#[derive(Clone, Copy)]
struct Fence {
    marker: u8,
    len: usize,
}

fn fence_marker(line: &str, allow_trailing_text: bool) -> Option<Fence> {
    let trimmed = line.trim_start_matches([' ', '\t']);
    let indent = line.len() - trimmed.len();
    if indent > 3 {
        return None;
    }
    let bytes = trimmed.as_bytes();
    let marker = *bytes.first()?;
    if marker != b'`' && marker != b'~' {
        return None;
    }
    let len = count_marker(bytes, marker);
    if !allow_trailing_text && !trimmed[len..].trim().is_empty() {
        return None;
    }
    (len >= 3).then_some(Fence { marker, len })
}

fn count_marker(bytes: &[u8], marker: u8) -> usize {
    bytes.iter().take_while(|byte| **byte == marker).count()
}

fn push_masked_line(line: &str, out: &mut String) {
    out.extend(line.chars().map(|ch| if ch == '\n' { '\n' } else { ' ' }));
}
