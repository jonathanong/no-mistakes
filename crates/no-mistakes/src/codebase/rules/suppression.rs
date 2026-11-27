use super::RuleFinding;
use crate::codebase::ts_source::{DisableCommentIndex, SourceStore};
use no_mistakes_rule_kernel::suppression::retain_unsuppressed_with_index;
use std::path::Path;

mod accounting;
pub(crate) use accounting::read_directive_source;
pub use accounting::{
    suppress_domain_findings_with_source_files, suppress_domain_findings_with_source_locations,
    suppress_domain_findings_with_sources,
};
pub(crate) use no_mistakes_rule_kernel::suppression::{
    suppress_rule_findings_with_sources, suppress_rule_findings_with_sources_except,
};
pub use no_mistakes_rule_kernel::suppression::{
    SuppressedFinding, SuppressionDirective, SuppressionDirectiveKind, SuppressionTarget,
};

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

fn note_standalone_fallback() {
    if let Some(observer) = crate::diagnostics::current() {
        observer.increment("suppression.standalone_fallbacks", 1);
    }
}

#[cfg(test)]
pub(super) mod standalone;

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod invalid_utf8_tests;

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod index_tests;
