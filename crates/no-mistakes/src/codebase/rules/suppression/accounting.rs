use super::FindingSuppressions;
use crate::codebase::ts_source::{DisableDirective, SourceStore};
use serde::Serialize;
use std::path::Path;

/// A domain finding projected into the common suppression contract.
///
/// The check runner creates these only after all domain analyzers have consumed
/// the request-scoped `SourceStore`; this keeps suppression from introducing a
/// second source-read path.
pub struct SuppressionTarget<'a> {
    pub domain: &'static str,
    pub rule: &'a str,
    pub file: &'a str,
    pub line: Option<usize>,
    pub reason: &'a str,
    /// Internal identity used when multiple public findings share a source
    /// location. It is intentionally omitted from the serialized contract.
    pub identity: Option<&'a str>,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SuppressedFinding {
    pub domain: String,
    pub rule: String,
    pub file: String,
    /// File containing the directive when it differs from the diagnostic target.
    pub source_file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    /// The deterministic diagnostic that would have been emitted.
    pub reason: String,
    pub directive: SuppressionDirective,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SuppressionDirective {
    pub kind: SuppressionDirectiveKind,
    pub line: usize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SuppressionDirectiveKind {
    File,
    Line,
    NextLine,
}

/// Retain unsuppressed findings and return deterministic accounting for the
/// findings removed by a valid directive. A missing source is deliberately not
/// treated as a suppression: it must not hide a diagnostic.
pub fn suppress_domain_findings_with_sources<T>(
    root: &Path,
    findings: &mut Vec<T>,
    sources: &SourceStore,
    describe: impl Fn(&T) -> SuppressionTarget<'_>,
) -> Vec<SuppressedFinding> {
    suppress_domain_findings_with_source_files(root, findings, sources, describe, |_| None)
}

/// Retain findings while reading suppression directives from an internal
/// provenance path. The public finding location remains the target location.
#[doc(hidden)]
pub fn suppress_domain_findings_with_source_files<T>(
    root: &Path,
    findings: &mut Vec<T>,
    sources: &SourceStore,
    describe: impl Fn(&T) -> SuppressionTarget<'_>,
    source_file: impl Fn(&T) -> Option<&str>,
) -> Vec<SuppressedFinding> {
    suppress_domain_findings_with_source_locations(root, findings, sources, describe, |finding| {
        source_file(finding).map(|file| (file, None))
    })
}

/// Retain findings while reading directives from an internal provenance
/// location. The public finding location remains the target location, while
/// line-specific directives are matched against the provenance source line.
#[doc(hidden)]
pub fn suppress_domain_findings_with_source_locations<T>(
    root: &Path,
    findings: &mut Vec<T>,
    sources: &SourceStore,
    describe: impl Fn(&T) -> SuppressionTarget<'_>,
    source_location: impl Fn(&T) -> Option<(&str, Option<usize>)>,
) -> Vec<SuppressedFinding> {
    let mut sources = FindingSuppressions::new(root, Some(sources));
    let mut suppressed = Vec::new();
    findings.retain(|finding| {
        let target = describe(finding);
        let (source_file, source_line) =
            source_location(finding).unwrap_or((target.file, target.line));
        let Some(directive) = sources.matching(source_file, target.rule, source_line) else {
            return true;
        };
        suppressed.push(SuppressedFinding {
            domain: target.domain.to_string(),
            rule: target.rule.to_string(),
            file: target.file.to_string(),
            source_file: source_file.to_string(),
            line: target.line,
            reason: target.identity.map_or_else(
                || target.reason.to_string(),
                |identity| format!("{} (component {identity})", target.reason),
            ),
            directive: project(directive),
        });
        false
    });
    suppressed.sort();
    suppressed
}

fn project(directive: DisableDirective) -> SuppressionDirective {
    let (kind, line) = match directive {
        DisableDirective::File { line } => (SuppressionDirectiveKind::File, line),
        DisableDirective::Line { line } => (SuppressionDirectiveKind::Line, line),
        DisableDirective::NextLine { line } => (SuppressionDirectiveKind::NextLine, line),
    };
    SuppressionDirective {
        kind,
        line: line as usize,
    }
}

/// Auditing can read the lossy projection while strict reads retain their error.
pub(crate) fn read_directive_source(
    sources: &SourceStore,
    path: &Path,
) -> Option<std::sync::Arc<str>> {
    sources.read_source_path(path).ok()
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
#[path = "accounting_tests.rs"]
mod tests;
