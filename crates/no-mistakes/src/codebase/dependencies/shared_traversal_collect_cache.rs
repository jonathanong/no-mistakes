fn cached_traversal_entries(
    shared: &SharedTraversalContext,
    key: TraversalCacheKey,
    compute: impl FnOnce() -> Result<Vec<graph::NodeEntry>>,
) -> Result<(Vec<graph::NodeEntry>, Vec<crate::codebase::ts_resolver::TsConfigDiagnostic>)>
{
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
            let (entries, runtime_diagnostics) = shared
                .tsconfig_catalog
                .isolate_runtime_diagnostics(compute);
            entries.map(|entries| CachedTraversal {
                entries,
                runtime_diagnostics,
            })
            .map_err(|error| std::sync::Arc::<str>::from(format!("{error:#}")))
        })
        .clone()
        .map_err(|message| anyhow::anyhow!("{message}"))?;
    if !computed {
        shared.session.record_work("traversal.reuses", 1);
    }
    Ok((cached.entries, cached.runtime_diagnostics))
}
