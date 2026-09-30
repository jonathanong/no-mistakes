/// Runs `per_file` over every indexable file that has facts, in file order.
fn resolve_files<T: Send>(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    indexes: &CallableResolutionIndexes,
    per_file: impl Fn(&CallSiteResolution<'_, '_>, &TsFileFacts) -> T + Sync,
) -> Vec<T> {
    use rayon::prelude::*;
    edge_inputs
        .graph_files
        .indexable()
        .par_iter()
        .filter_map(|path| {
            let file = facts.get_ts_facts(path)?;
            let index = indexes.file(facts, path)?;
            let resolution = CallSiteResolution {
                edge_inputs,
                facts,
                resolver,
                indexes,
                path,
                index: &index,
            };
            Some(per_file(&resolution, file))
        })
        .collect()
}

fn collect_call_edges_for_core(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    callable_export_resolutions: &mut FxHashMap<
        (std::path::PathBuf, String),
        ExportedCallableResolution,
    >,
) -> (Vec<Edge>, Vec<ResolvedCallSite>) {
    let indexes = CallableResolutionIndexes::default();
    let per_file = resolve_files(
        edge_inputs,
        facts,
        resolver,
        &indexes,
        |resolution, file| {
            let mut output = (Vec::new(), Vec::new());
            for call in file
                .function_calls
                .iter()
                .filter(|call| is_traversable_call(resolution.index, call))
            {
                let (edge, site) = resolution.resolve(call);
                output.0.extend(edge);
                output.1.push(site);
            }
            output.1.extend(resolution.unknown_sites(file));
            output
        },
    );
    populate_callable_export_resolutions(
        edge_inputs,
        facts,
        resolver,
        &indexes,
        callable_export_resolutions,
    );
    per_file.into_iter().fold(
        (Vec::new(), Vec::new()),
        |(mut edges, mut sites), (file_edges, file_sites)| {
            edges.extend(file_edges);
            sites.extend(file_sites);
            (edges, sites)
        },
    )
}
