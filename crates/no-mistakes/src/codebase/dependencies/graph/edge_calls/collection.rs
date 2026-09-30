/// One resolution pass over every indexable file, split by what the plan
/// asked for: `Call` edges and call sites for `calls`, `Extends` edges and
/// class declarations for `extends`. A class base is never a call.
#[derive(Default)]
struct CallPass {
    edges: Vec<Edge>,
    sites: Vec<ResolvedCallSite>,
    classes: Vec<ClassDeclaration>,
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
            }
            if plan.extends {
                let (edges, classes) = resolution.class_bases(file);
                output.edges.extend(edges);
                output.classes.extend(classes);
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
    per_file
        .into_iter()
        .fold(CallPass::default(), |mut all, part| {
            all.edges.extend(part.edges);
            all.sites.extend(part.sites);
            all.classes.extend(part.classes);
            all
        })
}
