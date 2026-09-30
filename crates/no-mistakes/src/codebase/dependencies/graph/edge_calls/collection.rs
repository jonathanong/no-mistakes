/// Resolved calls and `extends` bases of every indexable source file.
#[derive(Default)]
struct ResolvedCalls {
    edges: Vec<Edge>,
    sites: Vec<ResolvedCallSite>,
    class_bases: Vec<ResolvedClassBase>,
}

impl ResolvedCalls {
    fn merge(mut self, mut other: Self) -> Self {
        self.edges.append(&mut other.edges);
        self.sites.append(&mut other.sites);
        self.class_bases.append(&mut other.class_bases);
        self
    }
}

fn collect_call_edges_for_core(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    callable_export_resolutions: &mut FxHashMap<
        (std::path::PathBuf, String),
        ExportedCallableResolution,
    >,
) -> ResolvedCalls {
    use rayon::prelude::*;
    // A call through `@scope/package` reaches the workspace package source, as
    // its import edge already does.
    let workspace_resolver =
        crate::codebase::ts_resolver::WorkspaceFallbackResolver::new(resolver, workspace);
    let resolver: &dyn ImportResolution = &workspace_resolver;
    let indexes = CallableResolutionIndexes::default();
    let output = edge_inputs
        .graph_files
        .indexable()
        .par_iter()
        .map(|path| {
            let Some(file) = facts.get_ts_facts(path) else {
                return ResolvedCalls::default();
            };
            let Some(index) = indexes.file(facts, path) else {
                return ResolvedCalls::default();
            };
            let resolution = CallSiteResolution {
                edge_inputs,
                facts,
                resolver,
                indexes: &indexes,
                path,
                index: &index,
            };
            let call_offsets = file
                .function_calls
                .iter()
                .filter(|call| is_traversable_call(&index, call))
                .map(|call| (call.caller_id, call.offset, call.invocation))
                .collect::<FxHashSet<_>>();
            let mut resolved = ResolvedCalls {
                class_bases: resolution.class_bases(file),
                ..ResolvedCalls::default()
            };
            for call in file
                .function_calls
                .iter()
                .filter(|call| is_traversable_call(&index, call))
            {
                let (edge, site) = resolution.resolve(call);
                resolved.edges.extend(edge);
                resolved.sites.push(site);
            }
            resolved.sites.extend(
                file.unknown_calls
                    .iter()
                    .filter(|call| {
                        !call_offsets.contains(&(call.caller_id, call.offset, call.invocation))
                    })
                    .map(|call| ResolvedCallSite {
                        file: path.to_path_buf(),
                        caller: call.caller.clone(),
                        caller_id: call.caller_id,
                        source_callee: "<unknown>".to_string(),
                        line: call.line,
                        offset: call.offset,
                        invocation: call.invocation,
                        target: ResolvedCallTarget::Unknown,
                    }),
            );
            resolved
        })
        .reduce(ResolvedCalls::default, ResolvedCalls::merge);
    populate_callable_export_resolutions(
        edge_inputs,
        facts,
        resolver,
        &indexes,
        callable_export_resolutions,
    );
    output
}
