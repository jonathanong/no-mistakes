use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::ts_resolver::TsConfigCatalog;
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;
use std::path::Path;
use std::sync::Arc;

/// Compatibility entrypoints can receive facts without runner projections.
/// Supplement only those config facts through the same request parser/store.
pub(crate) fn ensure(
    root: &Path,
    config: &NoMistakesConfig,
    catalog: &Arc<TsConfigCatalog>,
    shared: &CheckFactMap,
    session: &AnalysisSession,
    sources: &Arc<SourceStore>,
) -> Option<CheckFactMap> {
    let runner = crate::integration_tests::runner_config::prepare_dynamic_configs(
        root,
        config,
        shared.files(),
        Arc::clone(catalog),
        Arc::clone(sources),
    );
    let missing = runner
        .paths()
        .filter(|path| !shared.integration_runner_configs.contains_key(*path))
        .cloned()
        .collect::<Vec<_>>();
    if missing.is_empty() {
        return None;
    }
    let mut supplemented = shared.fact_view_with_supplemental(&CheckFactMap::default());
    runner.with_request_cache(None, || {
        for path in missing {
            if let Some(facts) = runner.parse_path_for_facts_with_session(session, &path) {
                supplemented.integration_runner_configs.insert(path, facts);
            }
        }
    });
    Some(supplemented)
}
