use pulldown_cmark::{Event, Options as MarkdownOptions, Parser, Tag};
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::markdown_link_display_text::parser::{markdown_links_outside_code, InlineLink};
use crate::mermaid_validation::{MermaidFence, MermaidFenceCollector};

#[derive(Clone, Copy, Default)]
struct FactDemand {
    pulldown: bool,
    display_links: bool,
}

#[derive(Default)]
pub(crate) struct MarkdownFactPlan {
    by_path: BTreeMap<PathBuf, FactDemand>,
}

impl MarkdownFactPlan {
    pub(crate) fn request_pulldown(&mut self, paths: impl IntoIterator<Item = PathBuf>) {
        for path in paths {
            self.by_path.entry(path).or_default().pulldown = true;
        }
    }

    pub(crate) fn request_display_links(&mut self, paths: impl IntoIterator<Item = PathBuf>) {
        for path in paths {
            self.by_path.entry(path).or_default().display_links = true;
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.by_path.is_empty()
    }
}

#[derive(Debug)]
pub(crate) struct MarkdownFacts {
    pub(crate) source: Arc<str>,
    pub(crate) line_count: usize,
    pub(crate) char_count: usize,
    pub(crate) table_count: usize,
    pub(crate) mermaid_count: usize,
    pub(crate) mermaid_fences: Vec<MermaidFence>,
    pub(crate) link_destinations: Vec<String>,
    pub(crate) display_links: Vec<InlineLink>,
}

#[derive(Default)]
pub(crate) struct MarkdownFactMap {
    by_path: BTreeMap<PathBuf, MarkdownFacts>,
}

impl MarkdownFactMap {
    pub(crate) fn prepare(
        plan: &MarkdownFactPlan,
        sources: &crate::codebase::ts_source::SourceStore,
    ) -> Self {
        if plan.is_empty() {
            return Self::default();
        }
        let mut facts = plan
            .by_path
            .par_iter()
            .filter_map(|(path, demand)| {
                let source = sources.read_path(path).ok()?;
                Some((path.clone(), collect(source, *demand)))
            })
            .collect::<Vec<_>>();
        facts.sort_unstable_by(|left, right| left.0.cmp(&right.0));
        Self {
            by_path: facts.into_iter().collect(),
        }
    }

    pub(crate) fn get(&self, path: &Path) -> Option<&MarkdownFacts> {
        self.by_path.get(path)
    }
}

fn collect(source: Arc<str>, demand: FactDemand) -> MarkdownFacts {
    let mut table_count = 0;
    let mut link_destinations = Vec::new();
    let mut mermaid_collector = MermaidFenceCollector::new(&source);
    if demand.pulldown {
        for (event, range) in Parser::new_ext(&source, MarkdownOptions::all()).into_offset_iter() {
            mermaid_collector.observe(&event, range);
            match event {
                Event::Start(Tag::Table(_)) => table_count += 1,
                Event::Start(Tag::Link { dest_url, .. }) => {
                    link_destinations.push(dest_url.into_string());
                }
                _ => {}
            }
        }
    }
    let mermaid_fences = mermaid_collector.finish();
    let mermaid_count = mermaid_fences.len();
    let display_links = if demand.display_links {
        markdown_links_outside_code(&source)
    } else {
        Vec::new()
    };
    MarkdownFacts {
        line_count: markdown_line_count(&source),
        char_count: source.chars().count(),
        source,
        table_count,
        mermaid_count,
        mermaid_fences,
        link_destinations,
        display_links,
    }
}

/// Mirrors `str::lines` for LF and CRLF while treating a lone CR as a line ending.
pub(crate) fn markdown_line_count(content: &str) -> usize {
    if content.is_empty() {
        return 0;
    }
    let bytes = content.as_bytes();
    let mut count = 0;
    let mut index = 0;
    let mut ends_with_line_ending = false;
    while index < bytes.len() {
        match bytes[index] {
            b'\n' => {
                count += 1;
                ends_with_line_ending = true;
            }
            b'\r' => {
                count += 1;
                ends_with_line_ending = true;
                if bytes.get(index + 1) == Some(&b'\n') {
                    index += 1;
                }
            }
            _ => ends_with_line_ending = false,
        }
        index += 1;
    }
    count + usize::from(!ends_with_line_ending)
}

#[cfg(test)]
#[path = "markdown_facts/tests.rs"]
mod tests;
