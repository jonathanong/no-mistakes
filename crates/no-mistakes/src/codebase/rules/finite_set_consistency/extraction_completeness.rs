use super::{extract, RuleFinding, RULE_ID};
use std::path::Path;

pub(super) fn has_unsuppressed_issues(
    root: &Path,
    set: &extract::ExtractedSet,
    sources: &crate::codebase::ts_source::SourceStore,
) -> bool {
    let mut findings = set
        .issues
        .iter()
        .map(|issue| RuleFinding {
            rule: RULE_ID.to_string(),
            file: issue.file.clone(),
            line: issue.line,
            message: issue.message.clone(),
            import: None,
            target: issue.target.clone(),
        })
        .collect::<Vec<_>>();
    super::super::suppress_rule_findings_with_sources(root, &mut findings, sources);
    !findings.is_empty()
}
