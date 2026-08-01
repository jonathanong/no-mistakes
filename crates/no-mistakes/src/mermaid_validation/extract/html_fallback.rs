use std::ops::Range;

use super::{is_mermaid_info, line_number, MermaidFence};

#[derive(Clone, Copy)]
struct OpeningFence {
    marker: u8,
    length: usize,
    body_start: usize,
    is_mermaid: bool,
}

pub(super) fn extract(source: &str, range: Range<usize>) -> Vec<MermaidFence> {
    let limit = range.end.min(source.len());
    let mut cursor = range.start.min(limit);
    let mut fences = Vec::new();

    while cursor < limit {
        let line_end = line_content_end(source, cursor, limit);
        let Some(opening) = opening_fence(source, cursor, line_end) else {
            cursor = next_line_start(source, line_end, limit);
            continue;
        };
        let closing = closing_fence(source, opening, limit);
        if opening.is_mermaid {
            let body_end = closing.map_or(limit, |(start, _)| start);
            fences.push(MermaidFence {
                content: source[opening.body_start..body_end].to_string(),
                fence_offset: cursor,
                fence_line: line_number(source, cursor),
                closed: closing.is_some(),
            });
        }
        let Some((_, closing_end)) = closing else {
            break;
        };
        cursor = closing_end;
    }

    fences
}

fn opening_fence(source: &str, start: usize, end: usize) -> Option<OpeningFence> {
    let line = &source.as_bytes()[start..end];
    let indent = line.iter().take_while(|byte| **byte == b' ').count();
    if indent > 3 || line.get(indent) == Some(&b'\t') {
        return None;
    }
    let marker = *line.get(indent)?;
    if marker != b'`' && marker != b'~' {
        return None;
    }
    let length = line[indent..]
        .iter()
        .take_while(|byte| **byte == marker)
        .count();
    if length < 3 {
        return None;
    }
    let info = &line[indent + length..];
    if marker == b'`' && info.contains(&b'`') {
        return None;
    }
    let info = std::str::from_utf8(info).expect("fence info must remain valid UTF-8");
    Some(OpeningFence {
        marker,
        length,
        body_start: next_line_start(source, end, source.len()),
        is_mermaid: is_mermaid_info(info),
    })
}

fn closing_fence(source: &str, opening: OpeningFence, limit: usize) -> Option<(usize, usize)> {
    let mut cursor = opening.body_start;
    while cursor < limit {
        let line_end = line_content_end(source, cursor, limit);
        let line = &source.as_bytes()[cursor..line_end];
        let indent = line.iter().take_while(|byte| **byte == b' ').count();
        if indent <= 3 {
            let remainder = &line[indent..];
            let length = remainder
                .iter()
                .take_while(|byte| **byte == opening.marker)
                .count();
            if length >= opening.length
                && remainder[length..]
                    .iter()
                    .all(|byte| byte.is_ascii_whitespace())
            {
                return Some((cursor, next_line_start(source, line_end, limit)));
            }
        }
        cursor = next_line_start(source, line_end, limit);
    }
    None
}

fn line_content_end(source: &str, start: usize, limit: usize) -> usize {
    source.as_bytes()[start..limit]
        .iter()
        .position(|byte| matches!(byte, b'\r' | b'\n'))
        .map_or(limit, |offset| start + offset)
}

fn next_line_start(source: &str, line_end: usize, limit: usize) -> usize {
    match source.as_bytes().get(line_end) {
        Some(b'\r') if source.as_bytes().get(line_end + 1) == Some(&b'\n') => {
            (line_end + 2).min(limit)
        }
        Some(b'\r' | b'\n') => (line_end + 1).min(limit),
        _ => line_end,
    }
}

#[cfg(test)]
#[path = "html_fallback/tests.rs"]
mod tests;
