use super::*;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_outcome::CheckFailure;
use crate::codebase::rules::postgres_sql_fragment_escape;

pub(super) const GRAPH_CONSUMERS: &[&str] = &[
    TEST_NO_UNMOCKED_DYNAMIC_IMPORTS,
    FORBIDDEN_CALLS,
    FORBIDDEN_DEPENDENCIES,
    REQUIRED_ENTRYPOINT_REACHABILITY,
    crate::codebase::rules::NO_CROSS_PACKAGE_RELATIVE_IMPORTS,
    UNCONSTRUCTED_ERROR_CLASS,
    "production-exports-only-used-by-tests",
    "coverage-producer-assertion-owner",
    "test-no-live-repository-scan",
    "query-reached-per-item",
    "no-single-enqueue-in-loop",
    "queue-rate-limit-error-delay",
    "postgres-no-mutable-sql-registry",
    "postgres-no-unbounded-sql-identifier",
    postgres_sql_fragment_escape::RULE_ID,
];

fn configured_graph_consumers(
    config: &crate::config::v2::NoMistakesConfig,
) -> impl Iterator<Item = &'static str> + '_ {
    GRAPH_CONSUMERS.iter().copied().filter(move |consumer| {
        if *consumer == postgres_sql_fragment_escape::RULE_ID {
            postgres_sql_fragment_escape::demands_analysis(config)
        } else {
            config.rule_configured(consumer)
        }
    })
}

pub(super) fn graph_failed(
    config: &crate::config::v2::NoMistakesConfig,
    session: &AnalysisSession,
    error: &anyhow::Error,
) {
    let source = error.downcast_ref::<crate::codebase::check_outcome::CheckInputFailure>();
    for consumer in configured_graph_consumers(config) {
        if consumer == postgres_sql_fragment_escape::RULE_ID
            && source.is_some_and(|failure| failure.file.is_some())
        {
            continue;
        }
        session.record_check_failure(CheckFailure {
            consumer: consumer.into(),
            file: source.and_then(|failure| failure.file.clone()),
            property: source.and_then(|failure| failure.property.clone()),
            message: format!("{error:#}"),
        });
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
    let mut consumers = None;
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
        let consumers = consumers.get_or_insert_with(|| {
            configured_graph_consumers(config)
                .filter(|consumer| *consumer != FORBIDDEN_DEPENDENCIES || !complete_imports)
                .collect::<Vec<_>>()
        });
        for &consumer in consumers.iter() {
            if consumer == postgres_sql_fragment_escape::RULE_ID {
                continue;
            }
            failures.push(CheckFailure {
                consumer: consumer.into(),
                file: Some(file.clone()),
                property: None,
                message: reason.clone(),
            });
        }
    }
    if consumers.is_some_and(|consumers| !consumers.is_empty()) {
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

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
#[path = "value_flow_prerequisite_tests.rs"]
mod budget_tests;
