use crate::finding::RuleFinding;
use crate::paths::normalize_discovery_path;
use crate::rule_ids::UNUSED_ALLOW_ENTRY;
use paths::{finding_source_candidate, markdown_prose};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod accounting;
mod paths;

pub use accounting::{
    suppress_domain_findings_with_source_files, suppress_domain_findings_with_source_locations,
    suppress_domain_findings_with_sources, SuppressedFinding, SuppressionDirective,
    SuppressionDirectiveKind, SuppressionTarget,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DisableDirective {
    File { line: u32 },
    Line { line: u32 },
    NextLine { line: u32 },
}

pub trait SuppressionIndex {
    fn matching(
        &self,
        rule: &str,
        line: Option<u32>,
        track_quotes: bool,
    ) -> Option<DisableDirective>;
}

impl<I: SuppressionIndex> SuppressionIndex for Arc<I> {
    fn matching(
        &self,
        rule: &str,
        line: Option<u32>,
        track_quotes: bool,
    ) -> Option<DisableDirective> {
        self.as_ref().matching(rule, line, track_quotes)
    }
}

/// Supplies the request's cached directive index for a validated candidate.
pub trait SuppressionSources {
    type Index: SuppressionIndex;

    fn load_index(&self, root: &Path, candidate: &Path, is_absolute: bool) -> Option<Self::Index>;

    const ALLOW_ABSOLUTE_PATHS: bool;
}

impl<S: SuppressionSources> SuppressionSources for Arc<S> {
    type Index = S::Index;
    const ALLOW_ABSOLUTE_PATHS: bool = S::ALLOW_ABSOLUTE_PATHS;

    fn load_index(&self, root: &Path, candidate: &Path, is_absolute: bool) -> Option<Self::Index> {
        self.as_ref().load_index(root, candidate, is_absolute)
    }
}

pub fn suppress_rule_findings_with_sources_except(
    root: &Path,
    findings: &mut Vec<RuleFinding>,
    sources: &impl SuppressionSources,
    already_suppressed_rules: &[&str],
) {
    let mut sources = FindingSuppressions::new(root, sources);
    findings.retain(|finding| {
        already_suppressed_rules.contains(&finding.rule.as_str())
            || sources
                .matching(&finding.file, &finding.rule, Some(finding.line))
                .is_none()
    });
}

pub fn suppress_rule_findings_with_sources(
    root: &Path,
    findings: &mut Vec<RuleFinding>,
    sources: &impl SuppressionSources,
) {
    suppress_rule_findings_with_sources_except(root, findings, sources, &[]);
}

pub fn retain_unsuppressed_with_index(
    findings: &mut Vec<RuleFinding>,
    index: &impl SuppressionIndex,
) {
    findings.retain(|finding| {
        finding.rule == UNUSED_ALLOW_ENTRY
            || index
                .matching(
                    &finding.rule,
                    finding.line.try_into().ok(),
                    !markdown_prose(&finding.file),
                )
                .is_none()
    });
}

struct FindingSuppressions<'a, S: SuppressionSources> {
    lexical_root: PathBuf,
    sources: &'a S,
    files: HashMap<String, Option<SuppressionFile<S::Index>>>,
}

struct SuppressionFile<I> {
    index: I,
    track_quotes: bool,
}

impl<'a, S: SuppressionSources> FindingSuppressions<'a, S> {
    fn new(root: &Path, sources: &'a S) -> Self {
        Self {
            lexical_root: normalize_discovery_path(root),
            sources,
            files: HashMap::new(),
        }
    }

    fn matching(
        &mut self,
        file: &str,
        rule: &str,
        line: Option<usize>,
    ) -> Option<DisableDirective> {
        if rule == UNUSED_ALLOW_ENTRY {
            return None;
        }
        let source = match self.files.get(file) {
            Some(cached) => cached,
            None => {
                let index = self.load(file);
                self.files.entry(file.to_owned()).or_insert(index)
            }
        }
        .as_ref()?;
        source.index.matching(
            rule,
            line.and_then(|line| line.try_into().ok()),
            source.track_quotes,
        )
    }

    fn load(&self, file: &str) -> Option<SuppressionFile<S::Index>> {
        let (candidate, is_absolute) =
            finding_source_candidate(&self.lexical_root, file, S::ALLOW_ABSOLUTE_PATHS)?;
        let index = self
            .sources
            .load_index(&self.lexical_root, &candidate, is_absolute)?;
        Some(SuppressionFile {
            index,
            track_quotes: !markdown_prose(file),
        })
    }
}
