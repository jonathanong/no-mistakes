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
    })
}

pub(super) fn boundary(req: &IndependentRuleRequest<'_>) -> Result<RuleChunk> {
    if !rule_enabled(req.config, SERVER_ROUTE_CLIENT_BOUNDARY) {
        return Ok(RuleChunk::from_findings(Vec::new()));
    }
    Ok(RuleChunk::from_findings(
        server_route_client_boundary::check_with_facts_for_aggregate(
            req.root,
            req.config,
            req.shared,
            req.inferred_roots,
            req.defer_suppression,
        )?,
    ))
}

pub(super) fn api_routes(req: &IndependentRuleRequest<'_>) -> Result<RuleChunk> {
    if !rule_enabled(req.config, NEXTJS_NO_API_ROUTES) {
        return Ok(RuleChunk::from_findings(Vec::new()));
    }
    Ok(RuleChunk::from_findings(
        nextjs_no_api_routes::check_with_facts_for_aggregate(
            req.root,
            req.config,
            req.shared,
            req.inferred_roots,
            req.defer_suppression,
        )?,
    ))
}

pub(super) fn caching(req: &IndependentRuleRequest<'_>) -> Result<RuleChunk> {
    if !rule_enabled(req.config, NEXTJS_NO_CACHING) {
        return Ok(RuleChunk::from_findings(Vec::new()));
    }
    Ok(RuleChunk::from_findings(
        nextjs_no_caching::check_with_facts_for_aggregate(
            req.root,
            req.config,
            req.shared,
            req.inferred_roots,
            req.defer_suppression,
        )?,
    ))
}

pub(super) fn storybook(req: &IndependentRuleRequest<'_>) -> Result<RuleChunk> {
    if !rule_enabled(req.config, REQUIRE_STORYBOOK_STORIES) {
        return Ok(RuleChunk::from_findings(Vec::new()));
    }
    Ok(RuleChunk::from_findings(storybook_findings(
        StorybookFindingsRequest {
            root: req.root,
            config: req.config,
            prepared_tsconfig_catalog: req.prepared_tsconfig_catalog,
            shared: req.shared,
            inferred_roots: req.inferred_roots,
            session: req.session,
            defer_suppression: req.defer_suppression,
            sources: req.sources,
        },
    )?))
}

pub(super) fn playwright(req: &IndependentRuleRequest<'_>) -> Result<RuleChunk> {
    if !crate::playwright::rules::configured(req.config) {
        return Ok(RuleChunk::from_findings(Vec::new()));
    }
    let findings =
        crate::perf_trace::trace("rules.playwright", || match req.prepared_playwright {
            Some(prepared) => crate::playwright::rules::check_with_prepared_facts(
                req.root,
                req.config_path,
                req.config,
                req.shared,
                prepared,
            ),
            None => crate::playwright::rules::check_with_facts(
                req.root,
                req.config_path,
                req.config,
                req.shared,
            ),
        })?;
    Ok(RuleChunk::from_findings(findings))
}

pub(super) fn graph_rules(req: &IndependentRuleRequest<'_>) -> Result<RuleChunk> {
    Ok(RuleChunk::from_findings(graph_rule_findings(
        req.root,
        req.config,
        req.config_path,
        req.shared,
        req.prepared_graph,
        req.dependency_graph,
        req.inferred_roots,
    )?))
}
