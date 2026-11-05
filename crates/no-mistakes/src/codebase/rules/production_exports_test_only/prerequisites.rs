use super::*;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_outcome::CheckFailure;
use std::path::PathBuf;

pub(super) fn record_incomplete(
    incomplete: &[(PathBuf, usize, String)],
    facts: &CheckFactMap,
    session: Option<&AnalysisSession>,
) -> Result<()> {
    for (file, _, reason) in incomplete {
        let source = facts.get_ts_facts(file);
        if source.is_some_and(|source| {
            source.parse_error.is_none()
                && source.operational_error.is_none()
                && source.symbols.is_some()
        }) {
            continue;
        }
        let reason = source
            .and_then(|source| {
                source
                    .operational_error
                    .as_deref()
                    .or(source.parse_error.as_deref())
            })
            .unwrap_or(reason);
        if let Some(session) = session.filter(|session| session.partial_checks_enabled()) {
            let mut failure = CheckFailure::new(RULE_ID, reason);
            failure.file = Some(file.clone());
            session.record_check_failure(failure);
        } else {
            anyhow::bail!(
                "{RULE_ID}: required source {} could not be analyzed: {reason}",
                file.display()
            );
        }
    }
    Ok(())
}
