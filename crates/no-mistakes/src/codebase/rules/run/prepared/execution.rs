use super::*;

fn storybook_findings(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    tsconfig_path: Option<&Path>,
    prepared_tsconfig: &crate::codebase::ts_resolver::TsConfig,
    shared: &crate::codebase::check_facts::CheckFactMap,
    inferred_roots: Option<&crate::codebase::config::InferredRoots>,
) -> Result<Vec<RuleFinding>> {
    match inferred_roots {
        Some(inferred_roots) => require_storybook_stories::check_with_prepared_facts_and_inferred(
            root,
            config,
            tsconfig_path,
            prepared_tsconfig,
            shared,
            inferred_roots,
        ),
        None => require_storybook_stories::check_with_prepared_facts(
            root,
            config,
            tsconfig_path,
            prepared_tsconfig,
            shared,
        ),
    }
}

fn suppress_findings(
    root: &Path,
    findings: &mut Vec<RuleFinding>,
    sources: Option<&crate::codebase::ts_source::SourceStore>,
) {
    match sources {
        Some(sources) => suppress_rule_findings_with_sources(root, findings, sources),
        None => suppress_rule_findings(root, findings),
    }
}

pub(super) fn run(
    inputs: PreparedRulesCheck<'_>,
    dependency_graph: Option<&DepGraph>,
) -> Result<Vec<RuleFinding>> {
    let PreparedRulesCheck {
        root,
        config_path,
        tsconfig_path,
        shared,
        prepared_playwright,
        config,
        prepared_graph,
        prepared_tsconfig,
        inferred_roots,
        sources,
    } = inputs;
    if !any_codebase_rule_enabled(config) {
        return Ok(Vec::new());
    }
    if let Some(forbidden_plan) = forbidden_dependencies::graph_plan(config) {
        let (required_facts, _) = match prepared_graph {
            Some(prepared) => crate::codebase::dependencies::graph::
                ts_fact_plan_and_context_for_plan_with_prepared(root, forbidden_plan, prepared),
            None => crate::codebase::dependencies::graph::
                ts_fact_plan_and_context_for_plan_with_config(root, forbidden_plan, config_path),
        };
        if !shared.graph_plan().covers(required_facts) {
            anyhow::bail!(
                "shared check facts are missing graph facts required by {FORBIDDEN_DEPENDENCIES}"
            );
        }
    }
    let owned_graph;
    let dependency_graph = if let Some(graph) = dependency_graph {
        Some(graph)
    } else if let Some(plan) = canonical_graph_plan(config) {
        owned_graph =
            crate::perf_trace::trace(
                "rules.canonical_dependency_graph",
                || match prepared_graph {
                    Some(prepared) => {
                        DepGraph::build_with_plan_file_list_prepared_config_and_check_facts(
                            root,
                            prepared_tsconfig,
                            plan,
                            shared.graph_file_universe().to_vec(),
                            config_path,
                            shared,
                            prepared,
                        )
                    }
                    None => DepGraph::build_with_plan_file_list_config_and_complete_check_facts(
                        root,
                        prepared_tsconfig,
                        plan,
                        shared.graph_file_universe().to_vec(),
                        config_path,
                        shared,
                    ),
                },
            )?;
        Some(&owned_graph)
    } else {
        None
    };
    let mut findings = Vec::new();
    if rule_enabled(config, TEST_NO_UNMOCKED_DYNAMIC_IMPORTS) {
        findings.extend(crate::perf_trace::trace(
            "rules.test_no_unmocked_dynamic_imports",
            || {
                test_no_unmocked_dynamic_imports::check_with_prepared_facts_and_graph(
                    root,
                    config,
                    prepared_tsconfig,
                    shared,
                    dependency_graph.expect("dynamic-import rule requires canonical graph"),
                )
            },
        )?);
    }
    if rule_enabled(config, SERVER_ROUTE_CLIENT_BOUNDARY) {
        let boundary_findings = match inferred_roots {
            Some(inferred_roots) => server_route_client_boundary::check_with_facts_and_inferred(
                root,
                config,
                shared,
                inferred_roots,
            ),
            None => server_route_client_boundary::check_with_facts(root, config, shared),
        }?;
        findings.extend(boundary_findings);
    }
    if rule_enabled(config, NEXTJS_NO_API_ROUTES) {
        let api_route_findings = match inferred_roots {
            Some(inferred_roots) => nextjs_no_api_routes::check_with_facts_and_inferred(
                root,
                config,
                shared,
                inferred_roots,
            ),
            None => nextjs_no_api_routes::check_with_facts(root, config, shared),
        }?;
        findings.extend(api_route_findings);
    }
    if rule_enabled(config, NEXTJS_NO_CACHING) {
        findings.extend(match inferred_roots {
            Some(inferred_roots) => nextjs_no_caching::check_with_facts_and_inferred(
                root,
                config,
                shared,
                inferred_roots,
            ),
            None => nextjs_no_caching::check_with_facts(root, config, shared),
        }?);
    }
    if rule_enabled(config, REQUIRE_STORYBOOK_STORIES) {
        findings.extend(storybook_findings(
            root,
            config,
            tsconfig_path,
            prepared_tsconfig,
            shared,
            inferred_roots,
        )?);
    }
    if crate::playwright::rules::configured(config) {
        findings.extend(crate::perf_trace::trace(
            "rules.playwright",
            || match prepared_playwright {
                Some(prepared) => crate::playwright::rules::check_with_prepared_facts(
                    root,
                    config_path,
                    config,
                    shared,
                    prepared,
                ),
                None => {
                    crate::playwright::rules::check_with_facts(root, config_path, config, shared)
                }
            },
        )?);
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
    suppress_findings(root, &mut findings, sources);
    sort_findings(&mut findings);
    Ok(findings)
}
