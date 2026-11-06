use crate::codebase::analysis_session::AnalysisSession;
use std::path::PathBuf;

pub(super) fn record_source_failures(
    session: &AnalysisSession,
    inputs: impl IntoIterator<Item = PathBuf>,
    failures: Vec<(PathBuf, String)>,
) -> anyhow::Result<()> {
    let mut inputs = inputs.into_iter().collect::<Vec<_>>();
    inputs.sort();
    inputs.dedup();
    session.register_check_demand("queues", inputs);

    let mut failures = failures;
    failures.sort_by(|left, right| left.0.cmp(&right.0));
    failures.dedup();
    if failures.is_empty() {
        return Ok(());
    }
    if !session.partial_checks_enabled() {
        anyhow::bail!("{}", failures[0].1);
    }
    for (file, message) in failures {
        session.record_check_source_failure(&file, &message);
    }
    Ok(())
}
