use super::{
    any_codebase_rule_enabled, forbidden_dependencies, nextjs_no_api_routes, nextjs_no_caching,
    require_storybook_stories, rule_enabled, server_route_client_boundary, sort_findings,
    suppress_rule_findings, test_no_unmocked_dynamic_imports, RuleFinding, FORBIDDEN_DEPENDENCIES,
    NEXTJS_NO_API_ROUTES, NEXTJS_NO_CACHING, REQUIRE_STORYBOOK_STORIES,
    SERVER_ROUTE_CLIENT_BOUNDARY, TEST_NO_UNMOCKED_DYNAMIC_IMPORTS,
};
use anyhow::Result;
use std::path::Path;

/// Preloaded inputs for the aggregate rules check.
///
/// This keeps the aggregate `check` path from reloading configuration while
/// leaving the standalone rule-check entry points unchanged.
#[doc(hidden)]
pub struct PreparedRulesCheck<'a> {
    pub root: &'a Path,
    pub config_path: Option<&'a Path>,
    pub tsconfig_path: Option<&'a Path>,
    pub shared: &'a crate::codebase::check_facts::CheckFactMap,
    pub prepared_playwright: Option<&'a crate::playwright::rules::PreparedPlaywrightRules>,
    pub config: &'a crate::config::v2::NoMistakesConfig,
    pub prepared_graph: Option<&'a crate::codebase::dependencies::graph::PreparedGraphConfig>,
    pub prepared_tsconfig: &'a crate::codebase::ts_resolver::TsConfig,
    pub inferred_roots: Option<&'a crate::codebase::config::InferredRoots>,
}

/// Shared-config entry point used by the aggregate `check` command.
#[doc(hidden)]
pub fn run_check_with_config_and_facts_and_playwright(
    inputs: PreparedRulesCheck<'_>,
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
    } = inputs;
    if !any_codebase_rule_enabled(config) {
        return Ok(Vec::new());
    }
    let mut findings = Vec::new();
    if rule_enabled(config, TEST_NO_UNMOCKED_DYNAMIC_IMPORTS) {
        findings.extend(crate::perf_trace::trace(
            "rules.test_no_unmocked_dynamic_imports",
            || {
                test_no_unmocked_dynamic_imports::check_with_prepared_facts(
                    root,
                    config,
                    prepared_tsconfig,
                    shared,
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
        let storybook_findings = match inferred_roots {
            Some(inferred_roots) => {
                require_storybook_stories::check_with_prepared_facts_and_inferred(
                    root,
                    config,
                    tsconfig_path,
                    prepared_tsconfig,
                    shared,
                    inferred_roots,
                )
            }
            None => require_storybook_stories::check_with_prepared_facts(
                root,
                config,
                tsconfig_path,
                prepared_tsconfig,
                shared,
            ),
        }?;
        findings.extend(storybook_findings);
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
            || match inferred_roots {
                Some(inferred_roots) => {
                    forbidden_dependencies::check_with_prepared_facts_and_inferred(
                        root,
                        config,
                        config_path,
                        prepared_tsconfig,
                        shared,
                        prepared_graph,
                        inferred_roots,
                    )
                }
                None => forbidden_dependencies::check_with_prepared_facts(
                    root,
                    config,
                    config_path,
                    prepared_tsconfig,
                    shared,
                    prepared_graph,
                ),
            },
        )?);
    }
    suppress_rule_findings(root, &mut findings);
    sort_findings(&mut findings);
    Ok(findings)
}
