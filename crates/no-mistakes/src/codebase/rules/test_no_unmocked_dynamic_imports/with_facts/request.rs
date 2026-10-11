use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::{
    collect_check_facts_with_graph_files_playwright_sources_and_session, CheckFactPlan,
};
use crate::codebase::ts_source::{discover_files_from_visible, VisiblePathSnapshot};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

pub(crate) fn check(
    root: &Path,
    config: &NoMistakesConfig,
    tsconfig_path: Option<&Path>,
) -> Result<Vec<super::RuleFinding>> {
    let session = AnalysisSession::new(crate::diagnostics::current());
    session.record_work("analysis.requests", 1);
    let visible = Arc::new(VisiblePathSnapshot::new(root));
    session.insert_visible_paths(root, Arc::clone(&visible));
    let files = discover_files_from_visible(
        root,
        &config.filesystem.skip_directories,
        &visible.paths_for(root),
    );
    let sources = visible.source_store_for(root);
    let (tsconfig, catalog) =
        super::tsconfig_catalog::for_files(root, config, tsconfig_path, &files, &sources)?;
    let catalog = Arc::new(catalog);
    let runner = crate::integration_tests::runner_config::prepare_dynamic_configs(
        root,
        config,
        &files,
        Arc::clone(&catalog),
        Arc::clone(&sources),
    );
    let facts = collect_check_facts_with_graph_files_playwright_sources_and_session(
        &session,
        root,
        (files.clone(), files),
        CheckFactPlan {
            imports: true,
            dynamic_imports: true,
            source: true,
            integration_runner_configs: Some(Arc::new(runner)),
            ..Default::default()
        },
        None,
        Arc::clone(&sources),
    );
    super::graph::check_with_prepared_facts_and_session(
        root, config, &tsconfig, &catalog, &facts, &session, &sources,
    )
}
