/// One resolution pass over every indexable file, split by what the plan
/// asked for: `Call` edges and call sites for `calls`, `Extends` edges and
/// class declarations for `extends`. A class base is never a call.
#[derive(Default)]
struct CallPass {
    edges: Vec<Edge>,
    sites: Vec<ResolvedCallSite>,
    classes: Vec<ClassDeclaration>,
    /// Namespaces whose use the graph cannot follow to a class, per file.
    escapes: Vec<(std::path::PathBuf, String)>,
}

fn collect_call_edges_for_core(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    callable_export_resolutions: &mut FxHashMap<
        (std::path::PathBuf, String),
        ExportedCallableResolution,
    >,
) -> CallPass {
    use rayon::prelude::*;
    let plan = edge_inputs.plan;
    let indexes = CallableResolutionIndexes::default();
    // Resolving a namespace member is part of `calls` as much as of `extends`:
    // a graph built for calls alone still needs `new Errors.X()` to be an edge.
    let namespaces_in_repo = (plan.calls || plan.extends)
        && edge_inputs.graph_files.indexable().iter().any(|path| {
            facts
                .get_ts_facts(path)
                .is_some_and(|file| !file.namespaces.roots.is_empty())
        });
    let per_file = edge_inputs
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
                indexes: &indexes,
                path,
                index: &index,
                namespaces_in_repo,
                escapes: Default::default(),
            };
            let mut output = CallPass::default();
            if plan.calls {
                for call in file
                    .function_calls
                    .iter()
                    .filter(|call| is_traversable_call(&index, call))
                {
                    let (edge, site) = resolution.resolve(call);
                    output.edges.extend(edge);
                    output.sites.push(site);
                }
                output.sites.extend(resolution.unknown_sites(file));
            } else if plan.extends && namespaces_in_repo {
                // No call edge is wanted, but a construction that cannot be
                // followed into a namespace still keeps its classes quiet.
                for call in file.function_calls.iter().filter(|call| {
                    call.invocation == InvocationKind::Construct
                        && call.callee.contains('.')
                        && is_traversable_call(&index, call)
                }) {
                    resolution.resolve(call);
                }
            }
            if plan.extends {
                let (edges, classes) = resolution.class_bases(file);
                output.edges.extend(edges);
                output.classes.extend(classes);
                if namespaces_in_repo {
                    resolution.record_namespace_escapes(file);
                    output.escapes.extend(resolution.escapes.take());
                }
            }
            Some(output)
        })
        .collect::<Vec<_>>();
    if plan.calls {
        populate_callable_export_resolutions(
            edge_inputs,
            facts,
            resolver,
            &indexes,
            callable_export_resolutions,
        );
    }
    let mut pass = per_file
        .into_iter()
        .fold(CallPass::default(), |mut all, part| {
            all.edges.extend(part.edges);
            all.sites.extend(part.sites);
            all.classes.extend(part.classes);
            all.escapes.extend(part.escapes);
            all
        });
    if namespaces_in_repo {
        let mut escapes = NamespaceEscapes::default();
        for (file, root) in pass.escapes.drain(..) {
            escapes.entry(file).or_default().insert(root);
        }
        mark_escaped_namespaces(&mut pass.classes, &escapes);
    }
    pass
}
