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
    error: &anyhow::Error,
) {
    for consumer in GRAPH_CONSUMERS {
        if config.rule_configured(consumer) {
            let source = error.downcast_ref::<crate::codebase::check_outcome::CheckInputFailure>();
            session.record_check_failure(CheckFailure {
                consumer: (*consumer).into(),
                file: source.and_then(|failure| failure.file.clone()),
                property: source.and_then(|failure| failure.property.clone()),
                message: format!("{error:#}"),
            });
        }
    }
}

pub(super) fn register_demands(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    shared: &crate::codebase::check_facts::CheckFactMap,
    session: &AnalysisSession,
    inferred: Option<&crate::codebase::config::InferredRoots>,
) -> Result<()> {
    let mut inferred = inferred.cloned().unwrap_or_default();
    for consumer in [NEXTJS_NO_CACHING] {
        for rule in config.rule_applications(consumer) {
            let inputs = match session.capture_check_result(
                consumer,
                crate::codebase::rules::nextjs_no_caching::SourceInputs::new(
                    root,
                    config,
                    rule,
                    &mut inferred,
                ),
            )? {
                Some(inputs) => inputs,
                None => continue,
            };
            session.register_check_demand(
                consumer,
                shared
                    .files()
                    .iter()
                    .filter(|file| inputs.contains(file))
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
    if try_canonical_graph_plan(config)
        .ok()
        .flatten()
        .is_some_and(|plan| plan.ts_fact_plan().needs_value_flow())
    {
        register_value_flow_failures(config, shared, session);
    }
    if !session.partial_checks_enabled() {
        if let Some(failure) = session.check_failures().first() {
            anyhow::bail!("{}: {}", failure.consumer, failure.message);
        }
    }
    Ok(())
}

fn register_value_flow_failures(
    config: &crate::config::v2::NoMistakesConfig,
    shared: &crate::codebase::check_facts::CheckFactMap,
    session: &AnalysisSession,
) {
    let mut failures = Vec::new();
    let complete_imports = forbidden_dependencies::eligible_partial_graph_plan(config).is_some();
    let files: std::collections::BTreeSet<_> = shared
        .files()
        .iter()
        .chain(shared.graph_file_universe())
        .collect();
    for file in files {
        let Some(reason) = shared
            .ts
            .get(file)
            .and_then(|facts| facts.ts.value_flow.incomplete_reason.as_ref())
        else {
            continue;
        };
        for consumer in GRAPH_CONSUMERS
            .iter()
            .filter(|consumer| config.rule_configured(consumer))
            .filter(|consumer| **consumer != FORBIDDEN_DEPENDENCIES || !complete_imports)
        {
            failures.push(CheckFailure {
                consumer: (*consumer).into(),
                file: Some(file.clone()),
                property: None,
                message: reason.clone(),
            });
        }
    }
    if !failures.is_empty() {
        if let Some(crate::codebase::check_outcome::Prerequisite::Failed(previous)) =
            session.check_prerequisite("codebase.canonical-graph")
        {
            failures.extend(previous);
        }
        session.set_check_prerequisite(
            "codebase.canonical-graph",
            crate::codebase::check_outcome::Prerequisite::Failed(failures),
        );
    }
}

#[cfg(test)]
#[path = "value_flow_prerequisite_tests.rs"]
mod budget_tests;
