use super::{runner::check_with_tsconfig, CheckFactMap, NoMistakesConfig, RuleFinding};
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::ts_resolver::TsConfig;
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub(crate) fn check_with_prepared_facts_and_session(
    root: &Path,
    config: &NoMistakesConfig,
    explicit_tsconfig_path: Option<&Path>,
    prepared_tsconfig: &TsConfig,
    shared: &CheckFactMap,
    session: &AnalysisSession,
) -> Result<Vec<RuleFinding>> {
    check_with_optional_inferred(
        root,
        config,
        explicit_tsconfig_path,
        prepared_tsconfig,
        shared,
        None,
        session,
    )
}

pub(crate) fn check_with_prepared_facts_and_inferred_and_session(
    root: &Path,
    config: &NoMistakesConfig,
    explicit_tsconfig_path: Option<&Path>,
    prepared_tsconfig: &TsConfig,
    shared: &CheckFactMap,
    inferred_roots: &crate::codebase::config::InferredRoots,
    session: &AnalysisSession,
) -> Result<Vec<RuleFinding>> {
    check_with_optional_inferred(
        root,
        config,
        explicit_tsconfig_path,
        prepared_tsconfig,
        shared,
        Some(inferred_roots),
        session,
    )
}

fn check_with_optional_inferred(
    root: &Path,
    config: &NoMistakesConfig,
    explicit_tsconfig_path: Option<&Path>,
    prepared_tsconfig: &TsConfig,
    shared: &CheckFactMap,
    inferred_roots: Option<&crate::codebase::config::InferredRoots>,
    session: &AnalysisSession,
) -> Result<Vec<RuleFinding>> {
    let mut automatic_tsconfigs: HashMap<PathBuf, TsConfig> = HashMap::new();
    check_with_tsconfig(
        root,
        config,
        shared,
        session,
        |project_root| {
            if explicit_tsconfig_path.is_some() {
                return Ok(prepared_tsconfig.clone());
            }
            if let Some(tsconfig) = automatic_tsconfigs.get(project_root) {
                return Ok(tsconfig.clone());
            }
            let tsconfig = crate::codebase::ts_resolver::resolve_tsconfig_from_visible(
                None,
                project_root,
                shared.files(),
            )?;
            automatic_tsconfigs.insert(project_root.to_path_buf(), tsconfig.clone());
            Ok(tsconfig)
        },
        inferred_roots,
    )
}
