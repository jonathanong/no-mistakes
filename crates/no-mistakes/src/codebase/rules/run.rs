use super::{
    nextjs_no_api_routes, nextjs_no_caching, require_storybook_stories, rule_enabled,
    server_route_client_boundary, sort_findings, test_no_unmocked_dynamic_imports, RuleFinding,
    NEXTJS_NO_API_ROUTES, NEXTJS_NO_CACHING, REQUIRE_STORYBOOK_STORIES,
    SERVER_ROUTE_CLIENT_BOUNDARY, TEST_NO_UNMOCKED_DYNAMIC_IMPORTS,
};
use anyhow::Result;
use std::path::Path;

pub fn run_check(
    root: &Path,
    config_path: Option<&Path>,
    tsconfig_path: Option<&Path>,
) -> Result<Vec<RuleFinding>> {
    let config = crate::config::v2::load_v2_config(root, config_path)?;
    if !any_codebase_rule_enabled(&config) {
        return Ok(Vec::new());
    }
    let mut findings = match rule_enabled(&config, TEST_NO_UNMOCKED_DYNAMIC_IMPORTS) {
        true => test_no_unmocked_dynamic_imports::check(root, &config, tsconfig_path)?,
        false => Vec::new(),
    };
    if rule_enabled(&config, SERVER_ROUTE_CLIENT_BOUNDARY) {
        findings.extend(server_route_client_boundary::check(root, &config)?);
    }
    if rule_enabled(&config, NEXTJS_NO_API_ROUTES) {
        findings.extend(nextjs_no_api_routes::check(root, &config)?);
    }
    if rule_enabled(&config, NEXTJS_NO_CACHING) {
        findings.extend(nextjs_no_caching::check(root, &config)?);
    }
    if rule_enabled(&config, REQUIRE_STORYBOOK_STORIES) {
        findings.extend(require_storybook_stories::check(
            root,
            &config,
            tsconfig_path,
        )?);
    }
    sort_findings(&mut findings);
    Ok(findings)
}

pub fn run_check_with_facts(
    root: &Path,
    config_path: Option<&Path>,
    tsconfig_path: Option<&Path>,
    shared: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let config = crate::config::v2::load_v2_config(root, config_path)?;
    if !any_codebase_rule_enabled(&config) {
        return Ok(Vec::new());
    }
    let mut findings = Vec::new();
    if rule_enabled(&config, TEST_NO_UNMOCKED_DYNAMIC_IMPORTS) {
        findings.extend(test_no_unmocked_dynamic_imports::check_with_facts(
            root,
            &config,
            tsconfig_path,
            shared,
        )?);
    }
    if rule_enabled(&config, SERVER_ROUTE_CLIENT_BOUNDARY) {
        findings.extend(server_route_client_boundary::check_with_facts(
            root, &config, shared,
        )?);
    }
    if rule_enabled(&config, NEXTJS_NO_API_ROUTES) {
        findings.extend(nextjs_no_api_routes::check_with_facts(
            root, &config, shared,
        )?);
    }
    if rule_enabled(&config, NEXTJS_NO_CACHING) {
        findings.extend(nextjs_no_caching::check_with_facts(root, &config, shared)?);
    }
    if rule_enabled(&config, REQUIRE_STORYBOOK_STORIES) {
        findings.extend(require_storybook_stories::check_with_facts(
            root,
            &config,
            tsconfig_path,
            shared,
        )?);
    }
    sort_findings(&mut findings);
    Ok(findings)
}

fn any_codebase_rule_enabled(config: &crate::config::v2::NoMistakesConfig) -> bool {
    rule_enabled(config, TEST_NO_UNMOCKED_DYNAMIC_IMPORTS)
        || rule_enabled(config, SERVER_ROUTE_CLIENT_BOUNDARY)
        || rule_enabled(config, NEXTJS_NO_API_ROUTES)
        || rule_enabled(config, NEXTJS_NO_CACHING)
        || rule_enabled(config, REQUIRE_STORYBOOK_STORIES)
}
