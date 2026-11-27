use crate::codebase::ts_source::SourceStore;
use no_mistakes_rule_kernel::suppression::{self, SuppressedFinding, SuppressionTarget};
use std::path::Path;

/// Retain unsuppressed findings and return deterministic accounting for the
/// findings removed by a valid directive. A missing source is deliberately not
/// treated as a suppression: it must not hide a diagnostic.
pub fn suppress_domain_findings_with_sources<T>(
    root: &Path,
    findings: &mut Vec<T>,
    sources: &SourceStore,
    describe: impl Fn(&T) -> SuppressionTarget<'_>,
) -> Vec<SuppressedFinding> {
    suppression::suppress_domain_findings_with_sources(root, findings, sources, describe)
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
    suppression::suppress_domain_findings_with_source_files(
        root,
        findings,
        sources,
        describe,
        source_file,
    )
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
    suppression::suppress_domain_findings_with_source_locations(
        root,
        findings,
        sources,
        describe,
        source_location,
    )
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
use super::{SuppressionDirective, SuppressionDirectiveKind};

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
