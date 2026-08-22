fn cache_traversal_error(error: anyhow::Error) -> std::sync::Arc<str> {
    std::sync::Arc::<str>::from(format!("{error:#}"))
}

fn replay_cached_traversal_error(message: std::sync::Arc<str>) -> anyhow::Error {
    anyhow::anyhow!("{message}")
}

fn cached_traversal_entries(
    shared: &SharedTraversalContext,
    key: TraversalCacheKey,
    compute: impl FnOnce() -> Result<(
        Vec<graph::NodeEntry>,
        Vec<crate::codebase::ts_resolver::TsConfigProvenance>,
    )>,
) -> Result<(
    Vec<graph::NodeEntry>,
    Vec<crate::codebase::ts_resolver::TsConfigDiagnostic>,
    Vec<crate::codebase::ts_resolver::TsConfigProvenance>,
)> {
    let cell = {
        let mut cache = shared
            .traversal_results
            .lock()
            .expect("traversal result cache is poisoned");
        cache
            .entry(key)
            .or_insert_with(|| std::sync::Arc::new(std::sync::OnceLock::new()))
            .clone()
    };
    let mut computed = false;
    let cached = cell
        .get_or_init(|| {
            computed = true;
            shared.session.record_work("traversal.computations", 1);
            let (computed, runtime_diagnostics) =
                shared.tsconfig_catalog.isolate_runtime_diagnostics(compute);
            computed
                .map(|(entries, tsconfig_provenance)| CachedTraversal {
                    entries,
                    runtime_diagnostics,
                    tsconfig_provenance,
                })
                .map_err(cache_traversal_error)
        })
        .clone()
        .map_err(replay_cached_traversal_error)?;
    if !computed {
        shared.session.record_work("traversal.reuses", 1);
    }
    Ok((
        cached.entries,
        cached.runtime_diagnostics,
        cached.tsconfig_provenance,
    ))
}
