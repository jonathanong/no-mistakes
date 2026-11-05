use super::*;

use crate::codebase::rules::test_no_live_repository_scan::Resolution;

mod request;
pub(super) use request::GraphRuleRequest;

pub(super) fn graph_rule_findings(
    request: GraphRuleRequest<'_>,
) -> Result<crate::codebase::rules::analysis_policy::AnalysisDiagnostics> {
    let GraphRuleRequest {
        root,
        config,
        config_path,
        shared,
        prepared_graph,
        dependency_graph,
        inferred_roots,
        sources,
        tsconfig_catalog,
        prepared_vitest_projects,
        prepared_playwright_projects,
    } = request;
    let mut findings = Vec::new();
    let mut advisories = Vec::new();
    if config.rule_configured(crate::codebase::rules::PRODUCTION_EXPORTS_ONLY_USED_BY_TESTS) {
        let diagnostics = crate::perf_trace::rule(
            "analysis.rules.production-exports-only-used-by-tests",
            || {
                crate::codebase::rules::production_exports_test_only::check_with_graph(
                    root,
                    config,
                    dependency_graph
                        .expect("production-exports-only-used-by-tests requires canonical graph"),
                    shared,
                )
            },
        )?;
        findings.extend(diagnostics.findings);
        advisories.extend(diagnostics.advisories);
    }
    if rule_enabled(config, FORBIDDEN_CALLS) {
        findings.extend(crate::perf_trace::rule(
            "analysis.rules.forbidden-calls",
            || {
                forbidden_calls::check_with_graph(
                    root,
                    config,
                    dependency_graph.expect("forbidden-calls requires canonical graph"),
                    prepared_vitest_projects,
                    prepared_playwright_projects,
                    shared.graph_file_universe(),
                    Some(shared),
                )
            },
        )?);
    }
    if rule_enabled(
        config,
        crate::codebase::rules::test_no_live_repository_scan::RULE_ID,
    ) {
        let diagnostics =
            crate::perf_trace::rule("analysis.rules.test-no-live-repository-scan", || {
                crate::codebase::rules::test_no_live_repository_scan::check_with_graph(
                    root,
                    config,
                    dependency_graph
                        .expect("test-no-live-repository-scan requires canonical graph"),
                    prepared_vitest_projects,
                    shared.graph_file_universe(),
                    shared,
                    &Resolution::new(tsconfig_catalog, prepared_graph),
                )
            })?;
        findings.extend(diagnostics.findings);
        advisories.extend(diagnostics.advisories);
    }
    if rule_enabled(config, FORBIDDEN_DEPENDENCIES) {
        findings.extend(crate::perf_trace::rule(
            "analysis.rules.forbidden-dependencies",
            || {
                forbidden_dependencies::check_with_prepared_facts_and_graph(
                    root,
                    config,
                    forbidden_dependencies::PreparedConfig {
                        path: config_path,
                        sources,
                    },
                    shared,
                    prepared_graph,
                    inferred_roots,
                    dependency_graph.expect("forbidden-dependencies requires canonical graph"),
                )
            },
        )?);
    }
    if rule_enabled(config, REQUIRED_ENTRYPOINT_REACHABILITY) {
        findings.extend(crate::perf_trace::rule(
            "analysis.rules.required-entrypoint-reachability",
            || {
                required_entrypoint_reachability::check_with_graph_and_inferred(
                    root,
                    config,
                    shared.graph_file_universe(),
                    dependency_graph
                        .expect("required-entrypoint-reachability requires canonical graph"),
                    inferred_roots,
                    sources,
                )
            },
        )?);
    }
    if rule_enabled(config, UNCONSTRUCTED_ERROR_CLASS) {
        let diagnostics =
            crate::perf_trace::rule("analysis.rules.unconstructed-error-class", || {
                unconstructed_error_class::check_with_graph(
                    root,
                    config,
                    dependency_graph.expect("unconstructed-error-class requires canonical graph"),
                )
            })?;
        findings.extend(diagnostics.findings);
        advisories.extend(diagnostics.advisories);
    }
    if rule_enabled(config, query_reached_per_item::RULE_ID) {
        findings.extend(crate::perf_trace::rule(
            "analysis.rules.query-reached-per-item",
            || {
                query_reached_per_item::check_with_graph(
                    root,
                    config,
                    dependency_graph.expect("query-reached-per-item requires canonical graph"),
                    shared,
                    sources,
                )
            },
        )?);
    }
    if rule_enabled(
        config,
        crate::codebase::rules::POSTGRES_NO_MUTABLE_SQL_REGISTRY,
    ) {
        findings.extend(crate::perf_trace::rule(
            "analysis.rules.postgres-no-mutable-sql-registry",
            || {
                crate::codebase::rules::postgres_no_mutable_sql_registry::check_with_graph(
                    root,
                    config,
                    dependency_graph,
                    shared,
                )
            },
        )?);
        findings.extend(
            crate::codebase::rules::postgres_undeclared_executors::check(
                root,
                config,
                crate::codebase::rules::POSTGRES_NO_MUTABLE_SQL_REGISTRY,
                shared.graph_file_universe(),
                shared,
            )?,
        );
    }
    if rule_enabled(
        config,
        crate::codebase::rules::POSTGRES_NO_UNBOUNDED_SQL_IDENTIFIER,
    ) {
        findings.extend(crate::perf_trace::rule(
            "analysis.rules.postgres-no-unbounded-sql-identifier",
            || {
                crate::codebase::rules::postgres_no_unbounded_sql_identifier::check_with_graph(
                    root,
                    config,
                    dependency_graph,
                    shared,
                )
            },
        )?);
        findings.extend(
            crate::codebase::rules::postgres_undeclared_executors::check(
                root,
                config,
                crate::codebase::rules::POSTGRES_NO_UNBOUNDED_SQL_IDENTIFIER,
                shared.graph_file_universe(),
                shared,
            )?,
        );
    }
    Ok(
        crate::codebase::rules::analysis_policy::AnalysisDiagnostics {
            findings,
            advisories,
            suppressed: Vec::new(),
        },
    )
}

#[cfg(test)]
mod tests;
