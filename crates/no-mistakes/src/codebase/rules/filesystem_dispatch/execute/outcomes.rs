use super::*;
use crate::codebase::analysis_session::AnalysisSession;

#[doc(hidden)]
pub fn run_filesystem_rules_with_config_snapshot_catalog_sources_facts_suppression_and_session<
    'a,
>(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    files: &[PathBuf],
    prepared: PreparedFilesystemRuleInputs<'a>,
    facts: Option<&crate::codebase::check_facts::CheckFactMap>,
    session: &'a AnalysisSession,
) -> Result<Vec<RuleFinding>> {
    let prepared = PreparedFilesystemRuleInputs {
        session: Some(session),
        ..prepared
    };
    next_config::run(root, config, files, prepared, facts, true)
}

pub(super) fn register_demand(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    consumer: &str,
    files: &[PathBuf],
    facts: Option<&crate::codebase::check_facts::CheckFactMap>,
    sources: &crate::codebase::ts_source::SourceStore,
    session: &AnalysisSession,
) -> Result<bool> {
    let demanded = super::source_demand::files(root, config, consumer, files, facts, sources)?;
    session.register_check_demand(consumer, demanded.iter().cloned());
    if let Some(facts) = facts {
        for file in &demanded {
            if let Some(message) = facts.ts.get(file).and_then(|facts| {
                crate::codebase::check_facts::sql_source_prerequisites::source_error(facts)
            }) {
                session.record_check_source_failure(file, message);
            }
        }
    }
    Ok(session.check_consumer_ready(consumer))
}

#[doc(hidden)]
pub fn run_filesystem_rules_with_config_snapshot_catalog_sources_facts_and_suppression(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    files: &[PathBuf],
    prepared: PreparedFilesystemRuleInputs<'_>,
    facts: Option<&crate::codebase::check_facts::CheckFactMap>,
) -> Result<Vec<RuleFinding>> {
    next_config::run(root, config, files, prepared, facts, true)
}
