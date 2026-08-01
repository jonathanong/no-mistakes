use pulldown_cmark::{Event, Options as MarkdownOptions, Parser, Tag};
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::markdown_link_display_text::parser::{markdown_links_outside_code, InlineLink};
#[cfg(feature = "mermaid-validation")]
use crate::mermaid_validation::MermaidFence;
use crate::mermaid_validation::MermaidFenceCollector;

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
    #[cfg(feature = "mermaid-validation")]
    pub(crate) mermaid_fences: Vec<MermaidFence>,
    pub(crate) link_destinations: Vec<String>,
    pub(crate) display_links: Vec<InlineLink>,
}

#[derive(Default)]
pub(crate) struct MarkdownFactMap {
    by_path: BTreeMap<PathBuf, Result<MarkdownFacts, Arc<io::Error>>>,
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
            .map(|(path, demand)| {
                let outcome = sources
                    .read_path(path)
                    .map(|source| collect(path, source, *demand));
                (path.clone(), outcome)
            })
            .collect::<Vec<_>>();
        facts.sort_unstable_by(|left, right| left.0.cmp(&right.0));
        Self {
            by_path: facts.into_iter().collect(),
        }
    }

    pub(crate) fn get_for_rule(
        &self,
        path: &Path,
        rule_id: &str,
    ) -> anyhow::Result<&MarkdownFacts> {
        match self.by_path.get(path) {
            Some(Ok(facts)) => Ok(facts),
            Some(Err(error)) => Err(anyhow::Error::new(Arc::clone(error)).context(format!(
                "{rule_id} could not read Markdown file {}: {error}. The rule cannot safely analyze incomplete Markdown facts; restore it as a readable UTF-8 file or exclude it from this rule",
                path.display()
            ))),
            None => anyhow::bail!(
                "{rule_id} could not analyze Markdown file {} because its facts were not prepared. This is an internal analysis-planning error; report it to no-mistakes",
                path.display()
            ),
        }
    }
}

fn collect(path: &Path, source: Arc<str>, demand: FactDemand) -> MarkdownFacts {
    let mut table_count = 0;
    let mut link_destinations = Vec::new();
    let mut mermaid_collector = if path.extension().is_some_and(|extension| extension == "mdx") {
        MermaidFenceCollector::new_with_mdx_html_fallback(&source)
    } else {
        MermaidFenceCollector::new(&source)
    };
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
        #[cfg(feature = "mermaid-validation")]
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
