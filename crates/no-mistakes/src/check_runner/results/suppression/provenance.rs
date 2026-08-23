use no_mistakes::codebase::rules::{
    suppress_domain_findings_with_source_files, RuleFinding, SuppressedFinding, SuppressionTarget,
};
use no_mistakes::codebase::ts_source::SourceStore;

struct RuleSuppressionEntry {
    finding: RuleFinding,
    source_file: Option<String>,
}

pub(super) fn suppress_rules_with_sources(
    root: &std::path::Path,
    sources: &SourceStore,
    findings: &mut Vec<RuleFinding>,
    source_files: &[Option<String>],
    domain: &'static str,
    suppressed: &mut Vec<SuppressedFinding>,
) {
    let mut entries = findings
        .drain(..)
        .enumerate()
        .map(|(index, finding)| RuleSuppressionEntry {
            finding,
            source_file: source_files.get(index).cloned().flatten(),
        })
        .collect::<Vec<_>>();
    suppressed.extend(suppress_domain_findings_with_source_files(
        root,
        &mut entries,
        sources,
        |entry| SuppressionTarget {
            domain,
            rule: &entry.finding.rule,
            file: &entry.finding.file,
            line: Some(entry.finding.line),
            reason: &entry.finding.message,
            identity: None,
        },
        |entry| entry.source_file.as_deref(),
    ));
    findings.extend(entries.into_iter().map(|entry| entry.finding));
}
