//! Audit Rust line documentation without making it an effective suppression.
use super::super::{DisableDirective, NamedDisableDirective};
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag};

pub(super) fn directives(comments: &str) -> Vec<NamedDisableDirective> {
    let mut found = Vec::new();
    let mut group = Vec::new();
    let mut kind = None;
    for (index, line) in comments.trim_start_matches('\u{FEFF}').lines().enumerate() {
        // The Rust lexer retained only genuine comments. A trailing doc
        // comment belongs to the following item, even after code on this line.
        let comment_start = line.find("//");
        let comment = comment_start.map(|start| &line[start..]);
        let doc = comment.and_then(|comment| {
            if comment.starts_with("///") && !comment.starts_with("////") {
                Some((b'/', &comment[3..]))
            } else {
                comment.strip_prefix("//!").map(|body| (b'!', body))
            }
        });
        let has_code = !line[..comment_start.unwrap_or(line.len())]
            .trim()
            .is_empty();
        // Rustdoc joins attributes across blank lines and ordinary comments;
        // those physical gaps must not terminate an open example fence.
        if has_code || doc.is_some_and(|(next, _)| kind != Some(next)) {
            collect(&group, &mut found);
            group.clear();
            kind = None;
        }
        if let Some((next, body)) = doc {
            kind = Some(next);
            group.push(((index + 1) as u32, body));
        }
    }
    collect(&group, &mut found);
    found
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
