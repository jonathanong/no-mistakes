use super::{AnalysisDiagnostics, DelegateRequest};
use crate::codebase::rules::{
    suppress_domain_findings_with_source_files, RuleFinding, SuppressionTarget,
};

struct EntryFinding<'a> {
    finding: RuleFinding,
    entry: &'a str,
}

pub(super) fn consume_entry_file_directive(
    request: &DelegateRequest<'_>,
    diagnostics: &mut AnalysisDiagnostics,
) {
    if !request.defer_suppression {
        return;
    }
    let entry = crate::codebase::ts_source::relative_slash_path(request.root, request.config_path);
    // The entry's historical file opt-out covers the whole delegated config.
    // A helper's line numbers must never match entry line/next-line directives.
    for (domain, findings) in [
        ("filesystem", &mut diagnostics.findings),
        ("advisories", &mut diagnostics.advisories),
    ] {
        let mut entries = findings
            .drain(..)
            .map(|finding| EntryFinding {
                finding,
                entry: entry.as_ref(),
            })
            .collect::<Vec<_>>();
        diagnostics
            .suppressed
            .extend(suppress_domain_findings_with_source_files(
                request.root,
                &mut entries,
                request.sources,
                |entry| SuppressionTarget {
                    domain,
                    rule: &entry.finding.rule,
                    file: &entry.finding.file,
                    line: Some(entry.finding.line),
                    reason: &entry.finding.message,
                    identity: None,
                },
                |entry| Some(entry.entry),
            ));
        findings.extend(entries.into_iter().map(|entry| entry.finding));
    }
}
