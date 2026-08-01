use std::ops::Range;

use super::{fence_syntax::is_closing_fence_suffix, is_mermaid_info, line_number, MermaidFence};

#[path = "html_fallback/container.rs"]
mod container;
use container::ContainerPrefix;
#[path = "html_fallback/mdx_expression.rs"]
mod mdx_expression;
pub(super) use mdx_expression::MdxExpressionScanner;
#[path = "html_fallback/mdx_jsx.rs"]
mod mdx_jsx;
pub(super) use mdx_jsx::looks_like_clear_mdx_jsx;

#[derive(Clone)]
struct OpeningFence {
    marker: u8,
    length: usize,
    body_start: usize,
    is_mermaid: bool,
    container: ContainerPrefix,
    can_interrupt_paragraph: bool,
}

pub(super) struct Extracted {
    pub(super) fences: Vec<MermaidFence>,
    pub(super) consumed_until: usize,
}

pub(super) fn extract(
    source: &str,
    range: Range<usize>,
    expressions: &mut MdxExpressionScanner,
) -> Extracted {
    let limit = range.end.min(source.len());
    let mut cursor = range.start.min(limit);
    let mut consumed_until = cursor;
    let mut fences = Vec::new();
    let mut in_paragraph = false;
    while cursor < limit {
        let line_end = line_content_end(source, cursor, limit);
        let opening = (!expressions.is_masking_markdown())
            .then(|| opening_fence(source, cursor, line_end))
            .flatten();
        let Some(opening) =
            opening.filter(|opening| !in_paragraph || opening.can_interrupt_paragraph)
        else {
            expressions.observe_line(&source.as_bytes()[cursor..line_end]);
            update_paragraph_state(&source.as_bytes()[cursor..line_end], &mut in_paragraph);
            cursor = next_line_start(source, line_end, limit);
            continue;
        };
        // A blank line can make pulldown-cmark end an MDX HTML-block range even
        // though any recognized fenced block remains open. Follow its actual
        // closer so a later range resumes after, rather than at, that closer.
        let closing = closing_fence(source, &opening, source.len());
        if opening.is_mermaid {
            let body_end = closing.map_or(limit, |(start, _)| start);
            fences.push(MermaidFence {
                content: body_content(source, &opening, body_end),
                fence_offset: cursor,
                fence_line: line_number(source, cursor),
                closed: closing.is_some(),
            });
        }
        let Some((_, closing_end)) = closing else {
            break;
        };
        cursor = closing_end;
        consumed_until = closing_end;
        in_paragraph = false;
    }

    Extracted {
        fences,
        consumed_until,
    }
}

fn opening_fence(source: &str, start: usize, end: usize) -> Option<OpeningFence> {
    let (line, container) = ContainerPrefix::from_opening_line(&source.as_bytes()[start..end]);
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
        can_interrupt_paragraph: container.can_interrupt_paragraph(),
        container,
    })
}

fn update_paragraph_state(line: &[u8], in_paragraph: &mut bool) {
    let first = line.iter().position(|byte| !matches!(byte, b' ' | b'\t'));
    match first.map(|index| line[index]) {
        None => *in_paragraph = false,
        Some(b'<') => {}
        Some(b'#') if is_atx_heading(line) => *in_paragraph = false,
        Some(_) => *in_paragraph = true,
    }
}

fn is_atx_heading(line: &[u8]) -> bool {
    let indent = line.iter().take_while(|byte| **byte == b' ').count();
    if indent > 3 {
        return false;
    }
    let hashes = line[indent..]
        .iter()
        .take_while(|byte| **byte == b'#')
        .count();
    (1..=6).contains(&hashes)
        && line
            .get(indent + hashes)
            .is_none_or(|byte| matches!(byte, b' ' | b'\t'))
}

fn closing_fence(source: &str, opening: &OpeningFence, limit: usize) -> Option<(usize, usize)> {
    let mut cursor = opening.body_start;
    while cursor < limit {
        let line_end = line_content_end(source, cursor, limit);
        let line = opening
            .container
            .strip_line(&source.as_bytes()[cursor..line_end])?;
        let indent = line.iter().take_while(|byte| **byte == b' ').count();
        if indent <= 3 {
            let remainder = &line[indent..];
            let length = remainder
                .iter()
                .take_while(|byte| **byte == opening.marker)
                .count();
            if length >= opening.length && is_closing_fence_suffix(&remainder[length..]) {
                return Some((cursor, next_line_start(source, line_end, limit)));
            }
        }
        cursor = next_line_start(source, line_end, limit);
    }
    None
}

fn body_content(source: &str, opening: &OpeningFence, end: usize) -> String {
    let mut content = String::new();
    let mut cursor = opening.body_start;
    while cursor < end {
        let line_end = line_content_end(source, cursor, end);
        let raw = &source.as_bytes()[cursor..line_end];
        let line = opening
            .container
            .strip_line(raw)
            .unwrap_or_else(|| raw.into());
        content.push_str(std::str::from_utf8(&line).expect("source line must remain valid UTF-8"));
        let next = next_line_start(source, line_end, end);
        content.push_str(&source[line_end..next]);
        cursor = next;
    }
    content
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
