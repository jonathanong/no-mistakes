use super::ReverseAnalysis;
use crate::codebase::dependencies::graph::SymbolIndex;
use crate::codebase::queries::shared::{ReversePrepared, Target};

/// Build the reverse import index for the whole project in one parallel scan.
/// Cheaper than a full `DepGraph` — it only resolves import/re-export edges.
pub(crate) fn build_reverse_analysis(target: &Target) -> anyhow::Result<ReverseAnalysis> {
    build_reverse_analysis_with_plan(
        target,
        crate::codebase::ts_source::facts::TsFactPlan::default(),
    )
}

/// Build reverse-import facts plus explicitly requested query-specific syntax
/// facts in the same project-wide parse pass.
pub(crate) fn build_reverse_analysis_with_plan(
    target: &Target,
    additional_plan: crate::codebase::ts_source::facts::TsFactPlan,
) -> anyhow::Result<ReverseAnalysis> {
    let mut plan = crate::codebase::ts_source::facts::TsFactPlan::imports_and_symbols();
    plan.include(additional_plan);
    let prepared = target.prepare_reverse()?;
    let facts =
        crate::codebase::ts_source::facts::collect_ts_facts_with_context_sources_and_session(
            &target.session,
            prepared.graph_files.indexable(),
            plan,
            &crate::codebase::ts_source::facts::TsFactContext::default(),
            &target.sources,
        );
    let (index, target_tsconfig) = build_reverse_index_from_prepared(target, &prepared, &facts);
    Ok(ReverseAnalysis {
        index,
        facts,
        target_tsconfig,
    })
}

/// Project already-collected facts through the ordinary reverse-query catalog.
/// Callers may share facts with a broader analysis, but the resolver/catalog
/// used for ordinary importer output remains the one prepared by `Target`.
pub(crate) fn build_reverse_index_from_prepared(
    target: &Target,
    prepared: &ReversePrepared,
    facts: &crate::codebase::ts_source::facts::TsFactMap,
) -> (SymbolIndex, crate::codebase::ts_resolver::TsConfig) {
    let target_tsconfig = prepared
        .tsconfig_catalog
        .config_for(&target.abs_file)
        .clone();
    let index = SymbolIndex::build_from_facts_workspace_resolution_cache_and_session(
        &target_tsconfig,
        Some(&prepared.tsconfig_catalog),
        &prepared.graph_files,
        facts,
        &prepared.workspace,
        None,
        &target.session,
    );
    (index, target_tsconfig)
}
