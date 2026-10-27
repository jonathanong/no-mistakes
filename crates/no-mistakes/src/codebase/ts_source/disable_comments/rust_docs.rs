//! Audit Rust documentation without making it an effective suppression.
use super::super::{DisableDirective, NamedDisableDirective};
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag};
use std::ops::Range;

pub(super) fn directives(
    comments: &str,
    source: &str,
    blocks: &[Range<usize>],
) -> Vec<NamedDisableDirective> {
    let mut docs = Vec::new();
    let mut offset = 0;
    for line in comments.split_inclusive('\n') {
        if let Some(start) = line.find("//") {
            let comment = &line[start..];
            if (comment.starts_with("///") && !comment.starts_with("////"))
                || comment.starts_with("//!")
            {
                docs.push((offset + start..offset + line.len(), comment.as_bytes()[2]));
            }
        }
        offset += line.len();
    }
    docs.extend(blocks.iter().map(|range| {
        let kind = if source.as_bytes()[range.start + 2] == b'!' {
            b'!'
        } else {
            b'/'
        };
        (range.clone(), kind)
    }));
    docs.sort_by_key(|(range, _)| range.start);
    let mut found = Vec::new();
    let mut group = Vec::new();
    let mut kind = None;
    let mut previous = 0;
    let mut line = 1;
    for (range, next) in docs {
        if kind != Some(next) || has_code(&comments[previous..range.start]) {
            collect(&group, &mut found);
            group.clear();
        }
        kind = Some(next);
        let text = &source[range.clone()];
        let block = text.starts_with("/*");
        let body = if block {
            text[3..].strip_suffix("*/").unwrap_or(&text[3..])
        } else {
            &text[3..]
        };
        line += source[previous..range.start]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count() as u32;
        for (index, body) in body.lines().enumerate() {
            let body = if block && index > 0 {
                body.trim_start().strip_prefix('*').unwrap_or(body)
            } else {
                body
            };
            group.push((line + index as u32, body));
        }
        line += text.bytes().filter(|byte| *byte == b'\n').count() as u32;
        previous = range.end;
    }
    collect(&group, &mut found);
    found
}

fn has_code(mut gap: &str) -> bool {
    loop {
        gap = gap.trim_start_matches(['\u{FEFF}', ' ', '\t', '\r', '\n']);
        if let Some(comment) = gap.strip_prefix("/*") {
            let Some(end) = comment.find("*/") else {
                return false;
            };
            gap = &comment[end + 2..];
        } else if let Some(comment) = gap.strip_prefix("//") {
            let Some(end) = comment.find('\n') else {
                return false;
            };
            gap = &comment[end..];
        } else {
            return !gap.is_empty();
        }
    }
}

fn collect(lines: &[(u32, &str)], found: &mut Vec<NamedDisableDirective>) {
    if !lines
        .iter()
        .any(|(_, body)| body.contains("no-mistakes-disable-"))
    {
        return;
    }
    let markdown = lines
        .iter()
        .map(|(_, body)| *body)
        .collect::<Vec<_>>()
        .join("\n");
    let mut code = Parser::new_ext(&markdown, Options::all())
        .into_offset_iter()
        .filter_map(|(event, range)| {
            matches!(
                event,
                Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(_)))
            )
            .then_some(range)
        })
        .peekable();
    let mut offset = 0;
    for &(line, body) in lines {
        let start = offset;
        offset += body.len() + 1;
        while code.peek().is_some_and(|range| range.end <= start) {
            code.next();
        }
        if code.peek().is_some_and(|range| range.start < offset) {
            continue;
        }
        let Some((suffix, rest)) = body
            .trim()
            .strip_prefix("no-mistakes-disable-")
            .and_then(|rest| rest.split_once(' '))
        else {
            continue;
        };
        let rule = rest
            .trim()
            .split(|ch: char| ch == ':' || ch.is_whitespace())
            .next()
            .unwrap_or("");
        if rule.is_empty() {
            continue;
        }
        let directive = match suffix {
            "file" => DisableDirective::File { line },
            "line" => DisableDirective::Line { line },
            "next-line" => DisableDirective::NextLine { line },
            _ => continue,
        };
        found.push(NamedDisableDirective {
            rule: rule.to_owned(),
            directive,
        });
    }
}
