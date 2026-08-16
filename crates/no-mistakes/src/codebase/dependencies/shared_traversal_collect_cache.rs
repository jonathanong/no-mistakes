fn cached_traversal_entries(
    shared: &SharedTraversalContext,
    key: TraversalCacheKey,
    compute: impl FnOnce() -> Result<Vec<graph::NodeEntry>>,
) -> Result<(Vec<graph::NodeEntry>, Vec<crate::codebase::ts_resolver::TsConfigDiagnostic>)>
{
    let (cell, inserted) = {
        let mut cache = shared
            .traversal_results
            .lock()
            .expect("traversal result cache is poisoned");
        match cache.entry(key) {
            std::collections::hash_map::Entry::Occupied(entry) => (entry.get().clone(), false),
            std::collections::hash_map::Entry::Vacant(entry) => (
                entry
                    .insert(std::sync::Arc::new(std::sync::OnceLock::new()))
                    .clone(),
                true,
            ),
        }
    };
    if !inserted {
        shared.session.record_work("traversal.reuses", 1);
    }
    let cached = cell
        .get_or_init(|| {
            if inserted {
                shared.session.record_work("traversal.computations", 1);
            }
            compute()
                .map(|entries| CachedTraversal {
                    entries,
                    runtime_diagnostics: shared.tsconfig_catalog.runtime_diagnostics(),
                })
                .map_err(|error| std::sync::Arc::<str>::from(format!("{error:#}")))
        })
        .clone()
        .map_err(|message| anyhow::anyhow!("{message}"))?;
    if !inserted {
        shared
            .tsconfig_catalog
            .replay_runtime_diagnostics(&cached.runtime_diagnostics);
    }
    Ok((cached.entries, cached.runtime_diagnostics))
}
