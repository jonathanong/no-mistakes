use super::{runner::check_with_resolver, CheckFactMap, NoMistakesConfig, RuleFinding};
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::ts_resolver::{normalize_path, ScopedImportResolver, TsConfigCatalog};
use anyhow::Result;
use std::collections::HashSet;
use std::path::Path;

pub(crate) fn check_with_prepared_facts_and_session(
    root: &Path,
    config: &NoMistakesConfig,
    prepared_tsconfig_catalog: &TsConfigCatalog,
    shared: &CheckFactMap,
    session: &AnalysisSession,
) -> Result<Vec<RuleFinding>> {
    check_with_optional_inferred(
        root,
        config,
        prepared_tsconfig_catalog,
        shared,
        None,
        session,
    )
}

pub(crate) fn check_with_prepared_facts_and_inferred_and_session(
    root: &Path,
    config: &NoMistakesConfig,
    prepared_tsconfig_catalog: &TsConfigCatalog,
    shared: &CheckFactMap,
    inferred_roots: &crate::codebase::config::InferredRoots,
    session: &AnalysisSession,
) -> Result<Vec<RuleFinding>> {
    check_with_optional_inferred(
        root,
        config,
        prepared_tsconfig_catalog,
        shared,
        Some(inferred_roots),
        session,
    )
}

fn check_with_optional_inferred(
    root: &Path,
    config: &NoMistakesConfig,
    prepared_tsconfig_catalog: &TsConfigCatalog,
    shared: &CheckFactMap,
    inferred_roots: Option<&crate::codebase::config::InferredRoots>,
    session: &AnalysisSession,
) -> Result<Vec<RuleFinding>> {
    let visible_files = shared
        .files()
        .iter()
        .map(|path| normalize_path(path))
        .collect::<HashSet<_>>();
    let resolver =
        ScopedImportResolver::new_in_session(prepared_tsconfig_catalog, &visible_files, session);
    check_with_resolver(root, config, shared, &resolver, inferred_roots)
}
