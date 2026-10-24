use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::config::v2::Unanalyzable;

pub(super) fn analyze(
    rel: &str,
    source: &str,
    assume_unspecified: bool,
    policy: Unanalyzable,
) -> AnalysisDiagnostics {
    super::checkout::analyze_with_default_branch(rel, source, assume_unspecified, policy, None)
}
