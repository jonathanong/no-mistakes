use super::graph_rules::graph_rule_findings;
use super::helpers::{storybook_findings, StorybookFindingsRequest};
use super::independent::{IndependentRuleRequest, RuleChunk};
use super::*;

pub(super) fn dynamic_imports(req: &IndependentRuleRequest<'_>) -> Result<RuleChunk> {
    if !rule_enabled(req.config, TEST_NO_UNMOCKED_DYNAMIC_IMPORTS) {
        return Ok(RuleChunk::from_findings(Vec::new()));
    }
    let findings = crate::perf_trace::trace("rules.test_no_unmocked_dynamic_imports", || {
        test_no_unmocked_dynamic_imports::check_with_prepared_facts_graph_and_session_with_suppression(
            test_no_unmocked_dynamic_imports::PreparedFactsGraphRequest {
                root: req.root,
                config: req.config,
                tsconfig_catalog: req.prepared_tsconfig_catalog,
                shared: req.shared,
                graph: req
                    .dependency_graph
                    .expect("dynamic-import rule requires canonical graph"),
                session: req.session,
                sources: req.sources,
                defer_suppression: req.defer_suppression,
            },
        )
    })?;
    Ok(RuleChunk {
        findings: findings.findings,
        suppression_sources: findings.suppression_sources,
        advisories: findings.advisories,
    })
}

pub(super) fn boundary(req: &IndependentRuleRequest<'_>) -> Result<RuleChunk> {
    if !rule_enabled(req.config, SERVER_ROUTE_CLIENT_BOUNDARY) {
        return Ok(RuleChunk::from_findings(Vec::new()));
    }
    crate::perf_trace::trace("rules.server_route_client_boundary", || {
        let output = server_route_client_boundary::diagnostics_with_facts_for_aggregate(
            req.root,
            req.config,
            req.shared,
            req.sources,
            req.inferred_roots,
            req.defer_suppression,
        )?;
        let mut chunk = RuleChunk::from_findings(output.findings);
        chunk.advisories = output.advisories;
        Ok(chunk)
    })
}

pub(super) fn api_routes(req: &IndependentRuleRequest<'_>) -> Result<RuleChunk> {
    if !rule_enabled(req.config, NEXTJS_NO_API_ROUTES) {
        return Ok(RuleChunk::from_findings(Vec::new()));
    }
    crate::perf_trace::trace("rules.nextjs_no_api_routes", || {
        Ok(RuleChunk::from_findings(
            nextjs_no_api_routes::check_with_facts_for_aggregate(
                req.root,
                req.config,
                req.shared,
                req.inferred_roots,
                req.defer_suppression,
            )?,
        ))
    })
}

pub(super) fn caching(req: &IndependentRuleRequest<'_>) -> Result<RuleChunk> {
    if !rule_enabled(req.config, NEXTJS_NO_CACHING) {
        return Ok(RuleChunk::from_findings(Vec::new()));
    }
    crate::perf_trace::trace("rules.nextjs_no_caching", || {
        let diagnostics = nextjs_no_caching::diagnostics_with_facts_for_aggregate(
            req.root,
            req.config,
            req.shared,
            req.inferred_roots,
            req.defer_suppression,
        )?;
        let mut chunk = RuleChunk::from_findings(diagnostics.findings);
        chunk.advisories = diagnostics.advisories;
        Ok(chunk)
    })
}

pub(super) fn storybook(req: &IndependentRuleRequest<'_>) -> Result<RuleChunk> {
    if !rule_enabled(req.config, REQUIRE_STORYBOOK_STORIES) {
        return Ok(RuleChunk::from_findings(Vec::new()));
    }
    crate::perf_trace::trace("rules.require_storybook_stories", || {
        let diagnostics = storybook_findings(StorybookFindingsRequest {
            root: req.root,
            config: req.config,
            prepared_tsconfig_catalog: req.prepared_tsconfig_catalog,
            shared: req.shared,
            inferred_roots: req.inferred_roots,
            session: req.session,
            defer_suppression: req.defer_suppression,
            sources: req.sources,
        })?;
        let mut chunk = RuleChunk::from_findings(diagnostics.findings);
        chunk.advisories = diagnostics.advisories;
        Ok(chunk)
    })
}

pub(super) fn playwright(req: &IndependentRuleRequest<'_>) -> Result<RuleChunk> {
    if !crate::playwright::rules::configured(req.config) {
        return Ok(RuleChunk::from_findings(Vec::new()));
    }
    let output = crate::perf_trace::trace("rules.playwright", || match req.prepared_playwright {
        Some(prepared) => crate::playwright::rules::check_with_prepared_facts_outcome(
            req.root,
            req.config_path,
            req.config,
            req.shared,
            prepared,
        ),
        None => crate::playwright::rules::check_with_facts_outcome(
            req.root,
            req.config_path,
            req.config,
            req.shared,
        ),
    })?;
    Ok(RuleChunk {
        suppression_sources: std::iter::repeat_n(None, output.findings.len()).collect(),
        findings: output.findings,
        advisories: output.advisories,
    })
}

pub(super) fn graph_rules(req: &IndependentRuleRequest<'_>) -> Result<RuleChunk> {
    let diagnostics = graph_rule_findings(graph_rules::GraphRuleRequest {
        root: req.root,
        config: req.config,
        config_path: req.config_path,
        shared: req.shared,
        prepared_graph: req.prepared_graph,
        dependency_graph: req.dependency_graph,
        inferred_roots: req.inferred_roots,
        sources: Some(req.sources),
        prepared_vitest_projects: req.prepared_vitest_projects,
        prepared_playwright_projects: req.prepared_playwright_projects,
    })?;
    let mut chunk = RuleChunk::from_findings(diagnostics.findings);
    chunk.advisories = diagnostics.advisories;
    if req
        .config
        .rule_configured(crate::codebase::rules::coverage_producer_assertion_owner::RULE_ID)
    {
        let diagnostics = crate::codebase::rules::coverage_producer_assertion_owner::check(
            req.root,
            req.config,
            req.dependency_graph
                .expect("coverage ownership requires canonical graph"),
            req.shared,
            req.sources,
        )?;
        chunk
            .suppression_sources
            .extend(std::iter::repeat_n(None, diagnostics.findings.len()));
        chunk.findings.extend(diagnostics.findings);
        chunk.advisories.extend(diagnostics.advisories);
    }

    Ok(chunk)
}
