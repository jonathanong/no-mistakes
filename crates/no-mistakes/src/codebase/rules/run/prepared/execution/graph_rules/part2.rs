use super::*;

pub(super) fn collect(
    request: &GraphRuleRequest<'_>,
    session: Option<&crate::codebase::analysis_session::AnalysisSession>,
    findings: &mut Vec<RuleFinding>,
    advisories: &mut Vec<RuleFinding>,
) -> Result<()> {
    let root = request.root;
    let config = request.config;
    let shared = request.shared;
    let dependency_graph = request.dependency_graph;
    if rule_enabled(config, query_reached_per_item::RULE_ID) {
        let consumer = "query-reached-per-item";
        if dependency_graph.is_some()
            || session.is_none_or(|session| session.check_consumer_ready(consumer))
        {
            let result = (|| -> Result<()> {
                let produced =
                    crate::perf_trace::rule("analysis.rules.query-reached-per-item", || {
                        query_reached_per_item::check_with_graph_and_session(
                            root,
                            config,
                            dependency_graph
                                .expect("query-reached-per-item requires canonical graph"),
                            shared,
                            request.sources,
                            session,
                        )
                    })?;
                for finding in produced {
                    if finding.causes.is_empty() {
                        findings.push(finding);
                    } else {
                        advisories.push(finding);
                    }
                }
                Ok(())
            })();
            if let Some(session) = session {
                session.capture_check_result(consumer, result)?;
            } else {
                result?;
            }
        }
    }
    collect_single_enqueue_in_loop(request, session, findings)?;
    if rule_enabled(
        config,
        crate::codebase::rules::POSTGRES_NO_MUTABLE_SQL_REGISTRY,
    ) {
        let consumer = "postgres-no-mutable-sql-registry";
        if dependency_graph.is_some()
            || session.is_none_or(|session| session.check_consumer_ready(consumer))
        {
            let result = (|| -> Result<()> {
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

                Ok(())
            })();
            if let Some(session) = session {
                session.capture_check_result(consumer, result)?;
            } else {
                result?;
            }
        }
    }
    super::fragment_escape::collect(request, session, findings)?;
    if rule_enabled(
        config,
        crate::codebase::rules::POSTGRES_NO_UNBOUNDED_SQL_IDENTIFIER,
    ) {
        let consumer = "postgres-no-unbounded-sql-identifier";
        if dependency_graph.is_some()
            || session.is_none_or(|session| session.check_consumer_ready(consumer))
        {
            let result = (|| -> Result<()> {
                findings.extend(crate::perf_trace::rule(
            "analysis.rules.postgres-no-unbounded-sql-identifier",
            || {
                crate::codebase::rules::postgres_no_unbounded_sql_identifier::check_with_graph(
                    root,
                    config,
                    dependency_graph,
                    shared,
                    request.sources,
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

                Ok(())
            })();
            if let Some(session) = session {
                session.capture_check_result(consumer, result)?;
            } else {
                result?;
            }
        }
    }
    Ok(())
}

fn collect_single_enqueue_in_loop(
    request: &GraphRuleRequest<'_>,
    session: Option<&crate::codebase::analysis_session::AnalysisSession>,
    findings: &mut Vec<RuleFinding>,
) -> Result<()> {
    let root = request.root;
    let config = request.config;
    let shared = request.shared;
    let dependency_graph = request.dependency_graph;
    if rule_enabled(config, no_single_enqueue_in_loop::RULE_ID) {
        let consumer = no_single_enqueue_in_loop::RULE_ID;
        if dependency_graph.is_some()
            || session.is_none_or(|session| session.check_consumer_ready(consumer))
        {
            let result = (|| -> Result<()> {
                findings.extend(crate::perf_trace::rule(
                    "analysis.rules.no-single-enqueue-in-loop",
                    || {
                        no_single_enqueue_in_loop::check_with_catalog(
                            root,
                            config,
                            dependency_graph,
                            shared,
                            no_single_enqueue_in_loop::module_catalog(
                                request.tsconfig_catalog,
                                request.sources,
                                session,
                            ),
                        )
                    },
                )?);
                Ok(())
            })();
            if let Some(session) = session {
                session.capture_check_result(consumer, result)?;
            } else {
                result?;
            }
        }
    }
    Ok(())
}
