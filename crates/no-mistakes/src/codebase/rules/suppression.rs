use super::RuleFinding;
use crate::codebase::ts_source::{DisableCommentIndex, DisableDirective, SourceStore};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod accounting;
mod paths;
pub(crate) use accounting::read_directive_source;
pub use accounting::{
    suppress_domain_findings_with_source_files, suppress_domain_findings_with_source_locations,
    suppress_domain_findings_with_sources, SuppressedFinding, SuppressionDirective,
    SuppressionDirectiveKind, SuppressionTarget,
};
use paths::{finding_source_candidate, markdown_prose, source_path_for_candidate};

pub(crate) fn suppress_rule_findings_with_sources_except(
    root: &Path,
    findings: &mut Vec<RuleFinding>,
    sources: &SourceStore,
    already_suppressed_rules: &[&str],
) {
    suppress_rule_findings_inner(root, findings, Some(sources), already_suppressed_rules);
}

pub(crate) fn suppress_rule_findings_with_sources(
    root: &Path,
    findings: &mut Vec<RuleFinding>,
    sources: &SourceStore,
) {
    suppress_rule_findings_inner(root, findings, Some(sources), &[]);
}

pub(crate) fn suppress_rule_findings_for_path(
    sources: &SourceStore,
    path: &Path,
    findings: &mut Vec<RuleFinding>,
) {
    if findings.is_empty() {
        return;
    }
    let Some(index) = sources.suppressions_for_path(path) else {
        return;
    };
    retain_unsuppressed_with_index(findings, &index);
}

pub(crate) fn suppress_rule_findings_with_source(findings: &mut Vec<RuleFinding>, source: &str) {
    note_standalone_fallback();
    let index = DisableCommentIndex::new(source);
    retain_unsuppressed_with_index(findings, &index);
}

fn retain_unsuppressed_with_index<S: AsRef<str>>(
    findings: &mut Vec<RuleFinding>,
    index: &DisableCommentIndex<S>,
) {
    findings.retain(|finding| {
        finding.rule == super::allow_audit::RULE_ID
            || index
                .matching(
                    &finding.rule,
                    finding.line.try_into().ok(),
                    !markdown_prose(&finding.file),
                )
                .is_none()
    });
}

fn note_standalone_fallback() {
    if let Some(observer) = crate::diagnostics::current() {
        observer.increment("suppression.standalone_fallbacks", 1);
    }
}

pub(super) fn suppress_rule_findings_inner(
    root: &Path,
    findings: &mut Vec<RuleFinding>,
    request_sources: Option<&SourceStore>,
    already_suppressed_rules: &[&str],
) {
    let mut sources = FindingSuppressions::new(root, request_sources);
    findings.retain(|finding| {
        already_suppressed_rules.contains(&finding.rule.as_str())
            || sources
                .matching(&finding.file, &finding.rule, Some(finding.line))
                .is_none()
    });
}

type SharedIndex = Arc<DisableCommentIndex<Arc<str>>>;

struct FindingSuppressions<'a> {
    lexical_root: PathBuf,
    canonical_root: Option<PathBuf>,
    sources: Option<&'a SourceStore>,
    files: HashMap<String, Option<SuppressionFile>>,
}

struct SuppressionFile {
    index: SharedIndex,
    track_quotes: bool,
}

impl<'a> FindingSuppressions<'a> {
    fn new(root: &Path, sources: Option<&'a SourceStore>) -> Self {
        let lexical_root = crate::codebase::ts_source::normalize_discovery_path(root);
        let canonical_root = sources
            .is_none()
            .then(|| std::fs::canonicalize(&lexical_root).ok())
            .flatten();
        Self {
            lexical_root,
            canonical_root,
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
        if rule == super::allow_audit::RULE_ID {
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

    fn load(&self, file: &str) -> Option<SuppressionFile> {
        let (candidate, is_absolute) =
            finding_source_candidate(&self.lexical_root, file, self.sources.is_some())?;
        let index = match self.sources {
            Some(sources) => {
                let path = if is_absolute {
                    sources.trusted_regular_path(&candidate)
                } else {
                    sources.validated_regular_path(&self.lexical_root, &candidate)
                }?;
                sources.suppressions_for_path(&path)?
            }
            None => {
                note_standalone_fallback();
                let path = source_path_for_candidate(self.canonical_root.as_deref()?, candidate)?;
                let source = Arc::<str>::from(std::fs::read_to_string(path).ok()?);
                Arc::new(DisableCommentIndex::new(source))
            }
        };
        Some(SuppressionFile {
            index,
            track_quotes: !markdown_prose(file),
        })
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod invalid_utf8_tests;

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod index_tests;
