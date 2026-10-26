use super::{AnalysisDiagnostics, DelegateRequest};
use crate::codebase::rules::{suppress_domain_findings_with_source_files, SuppressionTarget};

pub(super) fn consume_entry_file_directive(
    request: &DelegateRequest<'_>,
    diagnostics: &mut AnalysisDiagnostics,
) {
    if !request.defer_suppression {
        return;
    }
    let entry = request.config_path.to_string_lossy();
    // The entry's historical file opt-out covers the whole delegated config.
    // A helper's line numbers must never match entry line/next-line directives.
    for (domain, findings) in [
        ("filesystem", &mut diagnostics.findings),
        ("advisories", &mut diagnostics.advisories),
    ] {
        diagnostics
            .suppressed
            .extend(suppress_domain_findings_with_source_files(
                request.root,
                findings,
                request.sources,
                |finding| SuppressionTarget {
                    domain,
                    rule: &finding.rule,
                    file: &finding.file,
                    line: Some(finding.line),
                    reason: &finding.message,
                    identity: None,
                },
                |_| Some(entry.as_ref()),
            ));
    }
}
