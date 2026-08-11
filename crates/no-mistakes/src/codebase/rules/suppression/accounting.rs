use super::{finding_source_candidate, matching_directive};
use serde::Serialize;
use std::collections::HashMap;
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
    sources: &crate::codebase::ts_source::SourceStore,
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
    sources: &crate::codebase::ts_source::SourceStore,
    describe: impl Fn(&T) -> SuppressionTarget<'_>,
    source_file: impl Fn(&T) -> Option<&str>,
) -> Vec<SuppressedFinding> {
    let lexical_root = crate::codebase::ts_source::normalize_discovery_path(root);
    let mut cached_sources = HashMap::new();
    let mut suppressed = Vec::new();
    findings.retain(|finding| {
        let target = describe(finding);
        let source_file = source_file(finding).unwrap_or(target.file);
        let source = cached_sources
            .entry(source_file.to_string())
            .or_insert_with(|| {
                let (candidate, is_absolute) =
                    finding_source_candidate(&lexical_root, source_file, true)?;
                let path = if is_absolute {
                    sources.trusted_regular_path(&candidate)
                } else {
                    sources.validated_regular_path(&lexical_root, &candidate)
                }?;
                super::super::read_source(sources, &path)
            });
        let Some(directive) = source
            .as_deref()
            .and_then(|source| matching_directive(source, target.rule, target.line))
        else {
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
            directive,
        });
        false
    });
    suppressed.sort();
    suppressed
}
