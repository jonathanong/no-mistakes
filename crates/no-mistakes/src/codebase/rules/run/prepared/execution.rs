use super::*;

mod graph_rules;
mod helpers;
mod independent;
mod independent_collect;
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
        inferred_roots,
        sources: _,
    } = inputs;
    if !any_codebase_rule_enabled(config) {
        return Ok(PreparedRuleFindings {
            findings: Vec::new(),
            suppression_sources: Vec::new(),
        });
    }
    if let Some(graph_plan) = canonical_graph_plan(config)? {
        let (required_facts, _) =
            required_graph_facts(root, graph_plan, config_path, prepared_graph, &session);
        if !shared.graph_plan().covers(required_facts) {
            anyhow::bail!(
                "shared check facts are missing graph facts required by configured codebase rules"
            );
        }
    }
    let owned_graph;
    let dependency_graph = if let Some(graph) = dependency_graph {
        Some(graph)
    } else if let Some(plan) = canonical_graph_plan(config)? {
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
            )?;
        Some(&owned_graph)
    } else {
        None
    };
    let (mut findings, suppression_sources) =
        independent::collect(independent::IndependentRuleRequest {
            session: &session,
            root,
            config_path,
            shared,
            prepared_playwright,
            config,
            prepared_graph,
            prepared_tsconfig_catalog,
            inferred_roots,
            sources,
            dependency_graph,
            defer_suppression,
        })?;
    if !defer_suppression {
        suppress_findings(root, &mut findings, sources);
    }
    Ok(finalize_findings(findings, suppression_sources))
}
