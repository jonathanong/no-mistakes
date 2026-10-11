use super::*;
use crate::codebase::ts_resolver::TsConfig;

pub fn check_with_facts(
    root: &Path,
    config: &NoMistakesConfig,
    tsconfig_path: Option<&Path>,
    shared: &CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current());
    session.insert_visible_paths(
        root,
        std::sync::Arc::new(crate::codebase::ts_source::VisiblePathSnapshot::from_paths(
            root,
            shared.files(),
        )),
    );
    let sources = session.visible_paths(root).source_store_for(root);
    let (tsconfig, catalog) =
        tsconfig_catalog::for_request(root, config, tsconfig_path, shared, &sources)?;
    let catalog = Arc::new(catalog);
    let supplemented = runner_configs::ensure(root, config, &catalog, shared, &session, &sources);
    let shared = supplemented.as_ref().unwrap_or(shared);
    check_with_prepared_facts_and_session(
        root, config, &tsconfig, &catalog, shared, &session, &sources,
    )
}

#[doc(hidden)]
pub fn check_with_prepared_facts(
    root: &Path,
    config: &NoMistakesConfig,
    tsconfig: &TsConfig,
    shared: &CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let catalog = Arc::new(tsconfig_catalog::forced(root, tsconfig));
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current());
    session.insert_visible_paths(
        root,
        std::sync::Arc::new(crate::codebase::ts_source::VisiblePathSnapshot::from_paths(
            root,
            shared.files(),
        )),
    );
    let sources = session.visible_paths(root).source_store_for(root);
    let supplemented = runner_configs::ensure(root, config, &catalog, shared, &session, &sources);
    let shared = supplemented.as_ref().unwrap_or(shared);
    check_with_prepared_facts_and_session(
        root, config, tsconfig, &catalog, shared, &session, &sources,
    )
}
