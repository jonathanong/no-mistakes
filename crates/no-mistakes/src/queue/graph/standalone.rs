use super::{build_filter, build_prepared_report, build_report, queue_definitions, FileFacts};
use super::{queue_project_facts_from_ts, resolve_producers, resolve_workers};
use super::{
    InternalProducer, InternalWorker, Path, PathBuf, PreparedProjectReport, ProjectReport,
};
use std::collections::HashMap;

pub(super) fn analyze_project(
    root: &Path,
    tsconfig_path: Option<&Path>,
    filters: &[String],
    snapshot: crate::codebase::ts_source::VisiblePathSnapshot,
) -> anyhow::Result<ProjectReport> {
    analyze_project_inner(root, tsconfig_path, filters, snapshot, build_report)
}

pub(super) fn analyze_project_indexed(
    root: &Path,
    tsconfig_path: Option<&Path>,
    filters: &[String],
    snapshot: crate::codebase::ts_source::VisiblePathSnapshot,
) -> anyhow::Result<PreparedProjectReport> {
    analyze_project_inner(
        root,
        tsconfig_path,
        filters,
        snapshot,
        build_prepared_report,
    )
}

fn analyze_project_inner<T>(
    root: &Path,
    tsconfig_path: Option<&Path>,
    filters: &[String],
    snapshot: crate::codebase::ts_source::VisiblePathSnapshot,
    builder: impl FnOnce(
        &Path,
        Vec<InternalProducer>,
        Vec<InternalWorker>,
        &HashMap<PathBuf, FileFacts>,
    ) -> T,
) -> anyhow::Result<T> {
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current());
    session.insert_visible_paths(root, std::sync::Arc::new(snapshot));
    let dataset = session.dataset(root);
    let visible_paths = dataset.paths_for(root);
    let sources = dataset.sources_for(root);
    let visible_files =
        crate::codebase::ts_source::discover_files_from_visible(root, &[], &visible_paths);
    let visible_set = visible_files.iter().cloned().collect();
    // Keep an explicit config as an intentional override, but select aliases
    // from each workspace package for automatic standalone queue analysis.
    // `visible_paths` is frozen by the snapshot above, so catalog construction
    // does not rediscover or reread the automatic root config.
    let tsconfig_catalog = match tsconfig_path {
        Some(path) => {
            let tsconfig = crate::codebase::ts_resolver::resolve_tsconfig_from_visible_and_sources(
                Some(path),
                root,
                &visible_paths,
                &sources,
            )?;
            crate::codebase::ts_resolver::TsConfigCatalog::forced(root, tsconfig, None)
        }
        None => crate::codebase::ts_resolver::TsConfigCatalog::from_visible_and_sources(
            root,
            &[],
            &visible_paths,
            &sources,
        ),
    };
    let filter = build_filter(filters)?;
    let factory_names = dataset
        .config(None)
        .map(|config| config.queues.factories.clone())
        .unwrap_or_default();
    let files = visible_files
        .into_iter()
        .filter(|path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| {
                    crate::codebase::ts_source::TS_JS_EXTENSIONS.contains(&extension)
                })
        })
        .filter(|path| {
            filter
                .as_ref()
                .is_none_or(|f| f.is_match(path.strip_prefix(root).unwrap_or(path)))
        })
        .collect::<Vec<_>>();
    let mut context = crate::codebase::ts_source::facts::TsFactContext::new(root);
    context.queue_project_factory_names = factory_names;
    let ts_facts =
        crate::codebase::ts_source::facts::collect_ts_facts_with_context_sources_and_session(
            &session,
            &files,
            crate::codebase::ts_source::facts::TsFactPlan {
                queue_project: true,
                ..Default::default()
            },
            &context,
            &sources,
        );
    crate::invocation::check_timeout()?;
    let facts = queue_project_facts_from_ts(ts_facts, filter.as_ref(), root);

    let queue_defs = queue_definitions(&facts);
    let resolver = crate::codebase::ts_resolver::ScopedImportResolver::new_in_session(
        &tsconfig_catalog,
        &visible_set,
        &session,
    )
    .with_queue_compatibility(root);
    let remapper =
        crate::codebase::ts_source::FrozenPathRemapper::from_paths(facts.keys().cloned());
    let mut producers = resolve_producers(&facts, &queue_defs, &resolver, &remapper);
    let mut workers = resolve_workers(&facts, &queue_defs, &resolver, &remapper);
    if let Ok(config) = dataset.config(None) {
        let (lang_producers, lang_workers) = crate::queue::lang::language_queue_sites(
            root,
            &session,
            config.as_ref(),
            filter.as_ref(),
        );
        producers.extend(lang_producers);
        workers.extend(lang_workers);
    }
    Ok(builder(root, producers, workers, &facts))
}
