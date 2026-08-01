use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd};
use std::ops::Range;

#[path = "extract/delimiter.rs"]
mod delimiter;
#[path = "fence_syntax.rs"]
mod fence_syntax;
#[path = "extract/html_fallback.rs"]
mod html_fallback;
use delimiter::{line_number, opening_delimiter};
use fence_syntax::{has_closing_fence, FenceDelimiter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HtmlFallbackMode {
    Disabled,
    All,
    ClearMdxJsx,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MermaidFence {
    pub(crate) content: String,
    pub(crate) fence_offset: usize,
    pub(crate) fence_line: usize,
    pub(crate) closed: bool,
}

struct ActiveFence {
    content: String,
    fence_offset: usize,
    fence_line: usize,
    delimiter: FenceDelimiter,
}

/// Incrementally extracts Mermaid fences from an existing pulldown-cmark pass.
///
/// This lets request-scoped Markdown analysis collect Mermaid source together
/// with its other facts instead of reparsing a file for each rule.
pub(crate) struct MermaidFenceCollector<'source> {
    source: &'source str,
    html_fallback: HtmlFallbackMode,
    list_item_depth: usize,
    active: Option<ActiveFence>,
    fences: Vec<MermaidFence>,
}

impl<'source> MermaidFenceCollector<'source> {
    pub(crate) fn new(source: &'source str) -> Self {
        Self {
            source,
            html_fallback: HtmlFallbackMode::Disabled,
            list_item_depth: 0,
            active: None,
            fences: Vec::new(),
        }
    }

    pub(crate) fn new_with_mdx_html_fallback(source: &'source str) -> Self {
        Self {
            source,
            html_fallback: HtmlFallbackMode::All,
            list_item_depth: 0,
            active: None,
            fences: Vec::new(),
        }
    }

    pub(crate) fn new_with_automatic_mdx_html_fallback(source: &'source str) -> Self {
        Self {
            source,
            html_fallback: HtmlFallbackMode::ClearMdxJsx,
            list_item_depth: 0,
            active: None,
            fences: Vec::new(),
        }
    }

    pub(crate) fn observe(&mut self, event: &Event<'_>, range: Range<usize>) {
        match event {
            Event::Start(Tag::Item) => {
                self.list_item_depth += 1;
            }
            Event::Start(Tag::HtmlBlock)
                if self.html_fallback == HtmlFallbackMode::All
                    || (self.html_fallback == HtmlFallbackMode::ClearMdxJsx
                        && html_fallback::looks_like_clear_mdx_jsx(self.source, range.clone())) =>
            {
                self.fences
                    .extend(html_fallback::extract(self.source, range));
            }
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) if is_mermaid_info(info) => {
                if let Some(delimiter) =
                    opening_delimiter(self.source, range.start, self.list_item_depth > 0)
                {
                    self.active = Some(ActiveFence {
                        content: String::new(),
                        fence_offset: range.start,
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
                        fence_offset: active.fence_offset,
                        fence_line: active.fence_line,
                        closed: has_closing_fence(self.source, active.delimiter, range.end),
                    });
                }
            }
            Event::End(TagEnd::Item) => {
                self.list_item_depth = self.list_item_depth.saturating_sub(1);
            }
            _ => {}
        }
    }

    pub(crate) fn finish(mut self) -> Vec<MermaidFence> {
        self.fences.sort_by_key(|fence| fence.fence_offset);
        self.fences.dedup_by_key(|fence| fence.fence_offset);
        self.fences
    }
}

pub(crate) fn extract_mermaid_fences(source: &str) -> Vec<MermaidFence> {
    extract_with_collector(source, MermaidFenceCollector::new(source))
}

pub(crate) fn extract_mermaid_fences_with_mdx_html_fallback(source: &str) -> Vec<MermaidFence> {
    extract_with_collector(
        source,
        MermaidFenceCollector::new_with_mdx_html_fallback(source),
    )
}

pub(crate) fn extract_mermaid_fences_with_automatic_mdx_html_fallback(
    source: &str,
) -> Vec<MermaidFence> {
    extract_with_collector(
        source,
        MermaidFenceCollector::new_with_automatic_mdx_html_fallback(source),
    )
}

fn extract_with_collector<'source>(
    source: &'source str,
    mut collector: MermaidFenceCollector<'source>,
) -> Vec<MermaidFence> {
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

#[cfg(test)]
#[path = "extract/tests.rs"]
mod tests;
