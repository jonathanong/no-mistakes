use super::independent_collect::{
    api_routes, boundary, caching, dynamic_imports, graph_rules, playwright, storybook,
};
use super::*;
use crate::diagnostics::{with_observer, with_timing_kind, TimingKind};

pub(super) struct RuleChunk {
    pub findings: Vec<RuleFinding>,
    pub suppression_sources: Vec<Option<String>>,
}

impl RuleChunk {
    pub fn from_findings(findings: Vec<RuleFinding>) -> Self {
        let suppression_sources = std::iter::repeat_n(None, findings.len()).collect();
        Self {
            findings,
            suppression_sources,
        }
    }
}

pub(super) struct IndependentRuleRequest<'a> {
    pub session: &'a std::sync::Arc<crate::codebase::analysis_session::AnalysisSession>,
    pub root: &'a Path,
    pub config_path: Option<&'a Path>,
    pub shared: &'a crate::codebase::check_facts::CheckFactMap,
    pub prepared_playwright: Option<&'a crate::playwright::rules::PreparedPlaywrightRules>,
    pub config: &'a crate::config::v2::NoMistakesConfig,
    pub prepared_graph: Option<&'a crate::codebase::dependencies::graph::PreparedGraphConfig>,
    pub prepared_tsconfig_catalog: &'a crate::codebase::ts_resolver::TsConfigCatalog,
    pub inferred_roots: Option<&'a crate::codebase::config::InferredRoots>,
    pub sources: &'a crate::codebase::ts_source::SourceStore,
    pub dependency_graph: Option<&'a DepGraph>,
    pub defer_suppression: bool,
}

pub(super) fn collect(
    req: IndependentRuleRequest<'_>,
) -> Result<(Vec<RuleFinding>, Vec<Option<String>>)> {
    let observer = crate::diagnostics::current();
    let run = |collect: fn(&IndependentRuleRequest<'_>) -> Result<RuleChunk>| {
        with_observer(observer.clone(), || {
            with_timing_kind(TimingKind::Parallel, || collect(&req))
        })
    };
    let (dynamic, (boundary, (api, (caching, (storybook, (playwright, graph)))))) = rayon::join(
        || run(dynamic_imports),
        || {
            rayon::join(
                || run(boundary),
                || {
                    rayon::join(
                        || run(api_routes),
                        || {
                            rayon::join(
                                || run(caching),
                                || {
                                    rayon::join(
                                        || run(storybook),
                                        || rayon::join(|| run(playwright), || run(graph_rules)),
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

fn merge_chunks(chunks: [RuleChunk; 7]) -> Result<(Vec<RuleFinding>, Vec<Option<String>>)> {
    let mut findings = Vec::new();
    let mut suppression_sources = Vec::new();
    for chunk in chunks {
        findings.extend(chunk.findings);
        suppression_sources.extend(chunk.suppression_sources);
    }
    Ok((findings, suppression_sources))
}
