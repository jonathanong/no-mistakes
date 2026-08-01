use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd};
use std::ops::Range;

#[path = "fence_syntax.rs"]
mod fence_syntax;
use fence_syntax::{has_closing_fence, line_end_with_ending, FenceDelimiter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MermaidFence {
    pub(crate) content: String,
    pub(crate) fence_line: usize,
    pub(crate) closed: bool,
}

struct ActiveFence {
    content: String,
    fence_line: usize,
    delimiter: FenceDelimiter,
}

/// Incrementally extracts Mermaid fences from an existing pulldown-cmark pass.
///
/// This lets request-scoped Markdown analysis collect Mermaid source together
/// with its other facts instead of reparsing a file for each rule.
pub(crate) struct MermaidFenceCollector<'source> {
    source: &'source str,
    active: Option<ActiveFence>,
    fences: Vec<MermaidFence>,
}

impl<'source> MermaidFenceCollector<'source> {
    pub(crate) fn new(source: &'source str) -> Self {
        Self {
            source,
            active: None,
            fences: Vec::new(),
        }
    }

    pub(crate) fn observe(&mut self, event: &Event<'_>, range: Range<usize>) {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) if is_mermaid_info(info) => {
                if let Some(delimiter) = opening_delimiter(self.source, range.start) {
                    self.active = Some(ActiveFence {
                        content: String::new(),
                        fence_line: line_number(self.source, range.start),
                        delimiter,
                    });
                }
            }
            Event::Text(text) => {
                if let Some(active) = &mut self.active {
                    active.content.push_str(text);
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some(active) = self.active.take() {
                    self.fences.push(MermaidFence {
                        content: active.content,
                        fence_line: active.fence_line,
                        closed: has_closing_fence(self.source, active.delimiter, range.end),
                    });
                }
            }
            _ => {}
        }
    }

    pub(crate) fn finish(self) -> Vec<MermaidFence> {
        self.fences
    }
}

pub(crate) fn extract_mermaid_fences(source: &str) -> Vec<MermaidFence> {
    let mut collector = MermaidFenceCollector::new(source);
    for (event, range) in
        pulldown_cmark::Parser::new_ext(source, pulldown_cmark::Options::all()).into_offset_iter()
    {
        collector.observe(&event, range);
    }
    collector.finish()
}

fn is_mermaid_info(info: &str) -> bool {
    info.split_whitespace()
        .next()
        .is_some_and(|token| token.eq_ignore_ascii_case("mermaid"))
}

fn opening_delimiter(source: &str, offset: usize) -> Option<FenceDelimiter> {
    let line_start = source[..offset.min(source.len())]
        .rfind(['\n', '\r'])
        .map_or(0, |index| index + 1);
    let line_end = source[line_start..]
        .find(['\n', '\r'])
        .map_or(source.len(), |index| line_start + index);
    let line = fence_syntax::strip_blockquote_prefix(&source.as_bytes()[line_start..line_end]);

    for index in 0..line.len() {
        let marker = line[index];
        if marker != b'`' && marker != b'~' {
            continue;
        }
        let length = line[index..]
            .iter()
            .take_while(|byte| **byte == marker)
            .count();
        if length < 3 {
            continue;
        }
        let trailing = std::str::from_utf8(&line[index + length..]).ok()?;
        if is_mermaid_info(trailing) {
            let prefix = &line[..index];
            let container_indent = if prefix.iter().any(|byte| !byte.is_ascii_whitespace()) {
                index
            } else {
                0
            };
            return Some(FenceDelimiter {
                marker,
                length,
                container_indent,
                content_start: line_end_with_ending(source, line_end),
            });
        }
    }
    None
}

fn line_number(source: &str, offset: usize) -> usize {
    let bytes = source.as_bytes();
    let mut line = 1;
    let mut index = 0;
    let limit = offset.min(bytes.len());
    while index < limit {
        match bytes[index] {
            b'\n' => line += 1,
            b'\r' => {
                line += 1;
                if bytes.get(index + 1) == Some(&b'\n') {
                    index += 1;
                }
            }
            _ => {}
        }
        index += 1;
    }
    line
}

#[cfg(test)]
#[path = "extract/tests.rs"]
mod tests;
