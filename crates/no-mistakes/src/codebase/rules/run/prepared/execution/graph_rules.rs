use super::*;

pub(super) fn graph_rule_findings(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    config_path: Option<&Path>,
    shared: &crate::codebase::check_facts::CheckFactMap,
    prepared_graph: Option<&crate::codebase::dependencies::graph::PreparedGraphConfig>,
    dependency_graph: Option<&DepGraph>,
    inferred_roots: Option<&crate::codebase::config::InferredRoots>,
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
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
