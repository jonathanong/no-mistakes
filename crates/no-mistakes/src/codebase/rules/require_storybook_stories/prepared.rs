use super::{runner::check_with_resolver, CheckFactMap, NoMistakesConfig, RuleFinding};
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::ts_resolver::{normalize_path, ScopedImportResolver, TsConfigCatalog};
use anyhow::Result;
use std::collections::HashSet;
use std::path::Path;

pub(crate) struct PreparedStorybookCheck<'a> {
    pub(crate) root: &'a Path,
    pub(crate) config: &'a NoMistakesConfig,
    pub(crate) prepared_tsconfig_catalog: &'a TsConfigCatalog,
    pub(crate) shared: &'a CheckFactMap,
    pub(crate) inferred_roots: Option<&'a crate::codebase::config::InferredRoots>,
    pub(crate) session: &'a AnalysisSession,
    pub(crate) defer_suppression: bool,
    pub(crate) sources: &'a crate::codebase::ts_source::SourceStore,
}

pub(crate) fn check_with_prepared_facts_for_aggregate(
    input: PreparedStorybookCheck<'_>,
) -> Result<Vec<RuleFinding>> {
    check_with_optional_inferred(input)
}

fn check_with_optional_inferred(input: PreparedStorybookCheck<'_>) -> Result<Vec<RuleFinding>> {
    let PreparedStorybookCheck {
        root,
        config,
        prepared_tsconfig_catalog,
        shared,
        inferred_roots,
        session,
        defer_suppression,
        sources,
    } = input;
    let visible_files = shared
        .files()
        .iter()
        .map(|path| normalize_path(path))
        .collect::<HashSet<_>>();
    let resolver =
        ScopedImportResolver::new_in_session(prepared_tsconfig_catalog, &visible_files, session);
    check_with_resolver(
        root,
        config,
        shared,
        &resolver,
        inferred_roots,
        defer_suppression,
        sources,
    )
}
