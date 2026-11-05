use super::independent_collect::{
    api_routes, boundary, caching, dynamic_imports, graph_rules, playwright, storybook,
};
use super::*;
use crate::diagnostics::{with_observer, with_timing_kind, TimingKind};

pub(super) struct RuleChunk {
    pub findings: Vec<RuleFinding>,
    pub suppression_sources: Vec<Option<String>>,
    pub advisories: Vec<RuleFinding>,
}

impl RuleChunk {
    pub fn from_findings(findings: Vec<RuleFinding>) -> Self {
        let suppression_sources = std::iter::repeat_n(None, findings.len()).collect();
        Self {
            findings,
            suppression_sources,
            advisories: Vec::new(),
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct IndependentRuleRequest<'a> {
    pub session: &'a std::sync::Arc<crate::codebase::analysis_session::AnalysisSession>,
    pub root: &'a Path,
    pub config_path: Option<&'a Path>,
    pub shared: &'a crate::codebase::check_facts::CheckFactMap,
    pub prepared_playwright: Option<&'a crate::playwright::rules::PreparedPlaywrightRules>,
    pub config: &'a crate::config::v2::NoMistakesConfig,
    pub prepared_graph: Option<&'a crate::codebase::dependencies::graph::PreparedGraphConfig>,
    pub prepared_tsconfig_catalog: &'a crate::codebase::ts_resolver::TsConfigCatalog,
    pub prepared_vitest_projects: Option<&'a crate::codebase::rules::PreparedVitestProjectCatalog>,
    pub prepared_playwright_projects:
        Option<&'a crate::codebase::rules::PreparedPlaywrightProjectCatalog>,
    pub inferred_roots: Option<&'a crate::codebase::config::InferredRoots>,
    pub sources: &'a crate::codebase::ts_source::SourceStore,
    pub dependency_graph: Option<&'a DepGraph>,
    pub defer_suppression: bool,
}

pub(super) fn collect(req: IndependentRuleRequest<'_>) -> Result<RuleChunk> {
    let observer = crate::diagnostics::current();
    let run = |consumer: &str,
               collect: fn(&IndependentRuleRequest<'_>) -> Result<RuleChunk>|
     -> Result<RuleChunk> {
        with_observer(observer.clone(), || {
            with_timing_kind(TimingKind::Parallel, || {
                if consumer == TEST_NO_UNMOCKED_DYNAMIC_IMPORTS
                    && req.dependency_graph.is_none()
                    && !req.session.check_consumer_ready(consumer)
                {
                    return Ok(RuleChunk::from_findings(Vec::new()));
                }
                if !req.session.partial_checks_enabled()
                    || consumer == "graph-rules"
                    || consumer == "playwright"
                {
                    return Ok(req
                        .session
                        .capture_check_result(consumer, collect(&req))?
                        .unwrap_or_else(|| RuleChunk::from_findings(Vec::new())));
                }
                let mut chunks = Vec::new();
                for (index, application) in req
                    .config
                    .rule_applications(consumer)
                    .into_iter()
                    .enumerate()
                {
                    let mut application = application.clone();
                    application
                        .resolved_application_index
                        .get_or_insert(index + 1);
                    let config = crate::config::v2::NoMistakesConfig {
                        rules: vec![application],
                        ..req.config.clone()
                    };
                    let scoped = IndependentRuleRequest {
                        config: &config,
                        ..req
                    };
                    chunks.push(
                        req.session
                            .capture_check_result(consumer, collect(&scoped))?
                            .unwrap_or_else(|| RuleChunk::from_findings(Vec::new())),
                    );
                }
                merge_chunks(chunks)
            })
        })
    };
    let (dynamic, (boundary, (api, (caching, (storybook, (playwright, graph)))))) = rayon::join(
        || run(TEST_NO_UNMOCKED_DYNAMIC_IMPORTS, dynamic_imports),
        || {
            rayon::join(
                || run(SERVER_ROUTE_CLIENT_BOUNDARY, boundary),
                || {
                    rayon::join(
                        || run(NEXTJS_NO_API_ROUTES, api_routes),
                        || {
                            rayon::join(
                                || run(NEXTJS_NO_CACHING, caching),
                                || {
                                    rayon::join(
                                        || run(REQUIRE_STORYBOOK_STORIES, storybook),
                                        || {
                                            rayon::join(
                                                || run("playwright", playwright),
                                                || run("graph-rules", graph_rules),
                                            )
                                        },
                                    )
                                },
                            )
                        },
                    )
                },
            )
        },
    );
    merge_chunks([
        dynamic?,
        boundary?,
        api?,
        caching?,
        storybook?,
        playwright?,
        graph?,
    ])
}

fn merge_chunks(chunks: impl IntoIterator<Item = RuleChunk>) -> Result<RuleChunk> {
    let mut findings = Vec::new();
    let mut suppression_sources = Vec::new();
    let mut advisories = Vec::new();
    for chunk in chunks {
        findings.extend(chunk.findings);
        suppression_sources.extend(chunk.suppression_sources);
        advisories.extend(chunk.advisories);
    }
    Ok(RuleChunk {
        findings,
        suppression_sources,
        advisories,
    })
}
