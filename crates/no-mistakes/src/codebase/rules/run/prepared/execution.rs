use super::*;
use crate::codebase::check_outcome::Prerequisite;

mod graph_rules;
mod helpers;
mod independent;
mod independent_collect;
mod outcomes;
mod source_store;
use helpers::{finalize_findings, required_graph_facts, suppress_findings};

pub(super) fn run(
    inputs: PreparedRulesCheck<'_>,
    dependency_graph: Option<&DepGraph>,
    aggregate_sources: Option<&crate::codebase::ts_source::SourceStore>,
    defer_suppression: bool,
) -> Result<PreparedRuleFindings> {
    let provided_sources = aggregate_sources.or(inputs.sources);
    let fallback_sources = provided_sources
        .is_none()
        .then(|| source_store::for_request(&inputs));
    let sources = provided_sources
        .or(fallback_sources.as_deref())
        .expect("prepared rules source fallback is initialized");
    let PreparedRulesCheck {
        session,
        root,
        config_path,
        tsconfig_path: _,
        shared,
        prepared_playwright,
        config,
        prepared_graph,
        prepared_tsconfig,
        prepared_tsconfig_catalog,
        prepared_vitest_projects,
        prepared_playwright_projects,
        inferred_roots,
        sources: _,
    } = inputs;
    if !any_codebase_rule_enabled(config) {
        return Ok(PreparedRuleFindings {
            findings: Vec::new(),
            suppression_sources: Vec::new(),
            advisories: Vec::new(),
        });
    }
    outcomes::register_demands(root, config, shared, &session, inferred_roots)?;
    let mut graph_plan = match try_canonical_graph_plan(config) {
        Ok(plan) => Prerequisite::Ready(plan),
        Err(error) if session.partial_checks_enabled() => {
            outcomes::graph_failed(config, &session, &error);
            Prerequisite::Failed(session.check_failures())
        }
        Err(error) => return Err(error),
    };
    if let Prerequisite::Ready(Some(required_plan)) = &graph_plan {
        let (required_facts, _) =
            required_graph_facts(root, *required_plan, config_path, prepared_graph, &session);
        if !shared.graph_plan().covers(required_facts)
            || !crate::codebase::rules::forbidden_calls::argument_facts_covered(
                root, config, shared,
            )
        {
            let message =
                "shared check facts are missing graph facts required by configured codebase rules";
            if session.partial_checks_enabled() {
                outcomes::graph_failed(config, &session, &anyhow::anyhow!(message));
                graph_plan = Prerequisite::Failed(session.check_failures());
            } else {
                anyhow::bail!("{message}");
            }
        }
    }
    if let Some(Prerequisite::Failed(failures)) =
        session.check_prerequisite("codebase.canonical-graph")
    {
        graph_plan = Prerequisite::Failed(failures);
    }
    if matches!(graph_plan, Prerequisite::Failed(_))
        && session.partial_checks_enabled()
        && session.check_consumer_ready(FORBIDDEN_DEPENDENCIES)
    {
        if let Some(plan) = forbidden_dependencies::eligible_partial_graph_plan(config) {
            graph_plan = Prerequisite::Ready(Some(plan));
        }
    }
    let owned_graph;
    let dependency_graph = if matches!(graph_plan, Prerequisite::Failed(_)) {
        None
    } else if let Some(graph) = dependency_graph {
        Some(graph)
    } else if let Prerequisite::Ready(Some(plan)) = graph_plan {
        owned_graph =
            crate::perf_trace::trace(
                "rules.canonical_dependency_graph",
                || match prepared_graph {
                    Some(prepared) => DepGraph::build_with_prepared_check_facts_and_session(
                        crate::codebase::dependencies::graph::PreparedCheckFactGraphBuildRequest {
                            root,
                            tsconfig: prepared_tsconfig,
                            tsconfig_catalog: prepared_tsconfig_catalog,
                            plan,
                            files: shared.graph_file_universe().to_vec(),
                            config_path,
                            facts: shared,
                            prepared,
                        },
                        session.clone(),
                    ),
                    None => DepGraph::build_with_complete_check_facts_and_session(
                        crate::codebase::dependencies::graph::CompleteCheckFactGraphBuildRequest {
                            root,
                            tsconfig: prepared_tsconfig,
                            tsconfig_catalog: prepared_tsconfig_catalog,
                            plan,
                            files: shared.graph_file_universe().to_vec(),
                            config_path,
                            facts: shared,
                        },
                        session.clone(),
                    ),
                },
            );
        match owned_graph {
            Ok(ref graph) => Some(graph),
            Err(error) if session.partial_checks_enabled() => {
                outcomes::graph_failed(config, &session, &error);
                None
            }
            Err(error) => return Err(error),
        }
    } else {
        None
    };
    let independent::RuleChunk {
        mut findings,
        suppression_sources,
        mut advisories,
    } = independent::collect(independent::IndependentRuleRequest {
        session: &session,
        root,
        config_path,
        shared,
        prepared_playwright,
        config,
        prepared_graph,
        prepared_tsconfig_catalog,
        prepared_vitest_projects,
        prepared_playwright_projects,
        inferred_roots,
        sources,
        dependency_graph,
        defer_suppression,
    })?;
    if !defer_suppression {
        suppress_findings(root, &mut findings, sources);
        suppress_findings(root, &mut advisories, sources);
    }
    Ok(finalize_findings(findings, suppression_sources, advisories))
}
