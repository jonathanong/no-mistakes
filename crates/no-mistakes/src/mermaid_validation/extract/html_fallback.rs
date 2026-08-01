use std::ops::Range;

use super::{is_mermaid_info, line_number, MermaidFence};

#[path = "html_fallback/container.rs"]
mod container;
use container::ContainerPrefix;

const STANDARD_HTML_TAGS: &str = concat!(
    "a abbr acronym address applet area article aside audio b base basefont bdi bdo big ",
    "blockquote body br button canvas caption center cite code col colgroup data datalist dd ",
    "del details dfn dialog dir div dl dt em embed fieldset figcaption figure font footer form ",
    "frame frameset h1 h2 h3 h4 h5 h6 head header hgroup hr html i iframe img input ins kbd ",
    "label legend li link main map mark marquee menu menuitem meta meter nav nobr noembed ",
    "noframes noscript object ol optgroup option output p param picture plaintext pre progress ",
    "q rb rp rt rtc ruby s samp script search section select slot small source span strike ",
    "strong style sub summary sup table tbody td template textarea tfoot th thead time title tr ",
    "track tt u ul var video wbr xmp",
);

#[derive(Clone)]
struct OpeningFence {
    marker: u8,
    length: usize,
    body_start: usize,
    is_mermaid: bool,
    container: ContainerPrefix,
}

pub(super) fn looks_like_clear_mdx_jsx(source: &str, range: Range<usize>) -> bool {
    let block = source[range].trim_start();
    let Some(after_open) = block.strip_prefix('<') else {
        return false;
    };
    if after_open.starts_with('>') {
        return true;
    }

    let name_end = after_open
        .find(|character: char| {
            !character.is_ascii_alphanumeric() && !matches!(character, '_' | '-' | '.' | ':' | '$')
        })
        .unwrap_or(after_open.len());
    let name = &after_open[..name_end];
    let component_name = name
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_uppercase())
        && !STANDARD_HTML_TAGS
            .split_ascii_whitespace()
            .any(|tag| name.eq_ignore_ascii_case(tag));
    component_name || has_unquoted_jsx_expression_brace(after_open)
}

fn has_unquoted_jsx_expression_brace(opening: &str) -> bool {
    let mut quote = None;
    for character in opening.chars() {
        if let Some(expected) = quote {
            if character == expected {
                quote = None;
            }
            continue;
        }
        match character {
            '\'' | '"' => quote = Some(character),
            '{' => return true,
            '>' => return false,
            _ => {}
        }
    }
    false
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
        let closing = closing_fence(source, &opening, limit);
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
    }

    fences
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
        container,
    })
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

fn body_content(source: &str, opening: &OpeningFence, end: usize) -> String {
    let mut content = String::new();
    let mut cursor = opening.body_start;
    while cursor < end {
        let line_end = line_content_end(source, cursor, end);
        let raw = &source.as_bytes()[cursor..line_end];
        let line = opening.container.strip_line(raw).unwrap_or(raw);
        content.push_str(std::str::from_utf8(line).expect("source line must remain valid UTF-8"));
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
