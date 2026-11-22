use super::*;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::rules::postgres_sql_fragment_escape as fragment_escape;

pub(super) fn collect(
    request: &GraphRuleRequest<'_>,
    session: Option<&AnalysisSession>,
    findings: &mut Vec<RuleFinding>,
) -> Result<()> {
    let config = request.config;
    let shared = request.shared;
    let root = request.root;
    let dependency_graph = request.dependency_graph;
    if rule_enabled(config, crate::codebase::rules::POSTGRES_SQL_FRAGMENT_ESCAPE) {
        let consumer = "postgres-sql-fragment-escape";
        if dependency_graph.is_some()
            || session.is_none_or(|session| session.check_consumer_ready(consumer))
            || session.is_some_and(|session| session.partial_checks_enabled())
        {
            let result = (|| -> Result<()> {
                let resolution = session
                    .zip(request.tsconfig_catalog.zip(request.prepared_graph))
                    .filter(|_| fragment_escape::demands_analysis(config))
                    .map(|(session, (catalog, prepared))| {
                        fragment_escape::FragmentImportResolution::new(
                            session,
                            catalog,
                            prepared.workspace(),
                            shared.graph_file_universe(),
                        )
                    });
                let report =
                    crate::perf_trace::rule("analysis.rules.postgres-sql-fragment-escape", || {
                        fragment_escape::check_with_graph(
                            root,
                            config,
                            dependency_graph,
                            shared,
                            request.sources,
                            resolution.as_ref(),
                        )
                    })?;
                if let Some(session) = session {
                    for gap in report.gaps {
                        session.record_check_failure(gap);
                    }
                }
                findings.extend(report.findings);
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
