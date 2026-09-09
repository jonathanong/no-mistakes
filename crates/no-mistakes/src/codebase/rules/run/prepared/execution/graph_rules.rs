use super::*;

pub(super) struct GraphRuleRequest<'a> {
    pub(super) root: &'a Path,
    pub(super) config: &'a crate::config::v2::NoMistakesConfig,
    pub(super) config_path: Option<&'a Path>,
    pub(super) shared: &'a crate::codebase::check_facts::CheckFactMap,
    pub(super) prepared_graph:
        Option<&'a crate::codebase::dependencies::graph::PreparedGraphConfig>,
    pub(super) dependency_graph: Option<&'a DepGraph>,
    pub(super) inferred_roots: Option<&'a crate::codebase::config::InferredRoots>,
    pub(super) prepared_vitest_projects:
        Option<&'a crate::codebase::rules::PreparedVitestProjectCatalog>,
    pub(super) prepared_playwright_projects:
        Option<&'a crate::codebase::rules::PreparedPlaywrightProjectCatalog>,
}

pub(super) fn graph_rule_findings(request: GraphRuleRequest<'_>) -> Result<Vec<RuleFinding>> {
    let GraphRuleRequest {
        root,
        config,
        config_path,
        shared,
        prepared_graph,
        dependency_graph,
        inferred_roots,
        prepared_vitest_projects,
        prepared_playwright_projects,
    } = request;
    let mut findings = Vec::new();
    if rule_enabled(config, FORBIDDEN_CALLS) {
        findings.extend(crate::perf_trace::trace("rules.forbidden_calls", || {
            forbidden_calls::check_with_graph(
                root,
                config,
                dependency_graph.expect("forbidden-calls requires canonical graph"),
                prepared_vitest_projects,
                prepared_playwright_projects,
                shared.graph_file_universe(),
            )
        })?);
    }
    if rule_enabled(config, FORBIDDEN_DEPENDENCIES) {
        findings.extend(crate::perf_trace::trace(
            "rules.forbidden_dependencies",
            || {
                forbidden_dependencies::check_with_prepared_facts_and_graph(
                    root,
                    config,
                    config_path,
                    shared,
                    prepared_graph,
                    inferred_roots,
                    dependency_graph.expect("forbidden-dependencies requires canonical graph"),
                )
            },
        )?);
    }
    if rule_enabled(config, REQUIRED_ENTRYPOINT_REACHABILITY) {
        findings.extend(crate::perf_trace::trace(
            "rules.required_entrypoint_reachability",
            || {
                required_entrypoint_reachability::check_with_graph_and_inferred(
                    root,
                    config,
                    shared.graph_file_universe(),
                    dependency_graph
                        .expect("required-entrypoint-reachability requires canonical graph"),
                    inferred_roots,
                )
            },
        )?);
    }
    Ok(findings)
}

#[cfg(test)]
mod tests;
