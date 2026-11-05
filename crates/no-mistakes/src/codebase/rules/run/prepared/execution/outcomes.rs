use super::*;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_outcome::CheckFailure;

pub(super) const GRAPH_CONSUMERS: &[&str] = &[
    TEST_NO_UNMOCKED_DYNAMIC_IMPORTS,
    FORBIDDEN_CALLS,
    FORBIDDEN_DEPENDENCIES,
    REQUIRED_ENTRYPOINT_REACHABILITY,
    UNCONSTRUCTED_ERROR_CLASS,
    "production-exports-only-used-by-tests",
    "coverage-producer-assertion-owner",
    "test-no-live-repository-scan",
    "query-reached-per-item",
    "postgres-no-mutable-sql-registry",
    "postgres-no-unbounded-sql-identifier",
];

pub(super) fn graph_failed(
    config: &crate::config::v2::NoMistakesConfig,
    session: &AnalysisSession,
    message: &str,
) {
    for consumer in GRAPH_CONSUMERS {
        if config.rule_configured(consumer) {
            session.record_check_failure(CheckFailure::new(*consumer, message));
        }
    }
}

pub(super) fn register_demands(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    shared: &crate::codebase::check_facts::CheckFactMap,
    session: &AnalysisSession,
) -> Result<()> {
    for consumer in [NEXTJS_NO_CACHING] {
        for rule in config.rule_applications(consumer) {
            let filter = match session.capture_check_result(
                consumer,
                crate::codebase::rules::path_filter::RulePathFilter::new(root, config, rule),
            )? {
                Some(filter) => filter,
                None => continue,
            };
            session.register_check_demand(
                consumer,
                shared
                    .files()
                    .iter()
                    .filter(|file| filter.is_match(file))
                    .cloned(),
            );
        }
    }
    for file in shared.files() {
        if let Some(error) = shared
            .ts
            .get(file)
            .and_then(|facts| facts.parse_error.as_deref())
        {
            session.record_check_source_failure(file, error);
        }
    }
    if !session.partial_checks_enabled() {
        if let Some(failure) = session.check_failures().first() {
            anyhow::bail!("{}: {}", failure.consumer, failure.message);
        }
    }
    Ok(())
}
