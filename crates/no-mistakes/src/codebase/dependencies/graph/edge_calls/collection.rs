/// One resolution pass over every indexable file, split by what the plan
/// asked for: `Call` edges and call sites for `calls`, `Extends` edges and
/// class declarations for `extends`. A class base is never a call.
#[derive(Default)]
struct CallPass {
    edges: Vec<Edge>,
    sites: Vec<ResolvedCallSite>,
    replacement_files: Vec<(
        std::path::PathBuf,
        std::sync::Arc<crate::codebase::dependencies::extract::ValueReplacementFacts>,
    )>,
    classes: Vec<ClassDeclaration>,
    class_ids: Vec<(std::path::PathBuf, FxHashSet<CallableId>)>,
    constructor_bases: Vec<(NodeId, NodeId)>,
    constructor_routes: value_flow::ConstructorRoutes,
    super_constructors: value_flow::ConstructorRoutes,
    parameter_cache: value_flow::ParameterCache,
    modules: Vec<(String, std::path::PathBuf)>,
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
    module_files: &mut FxHashMap<String, FxHashSet<std::path::PathBuf>>,
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
            if plan.calls && plan.effect_selectors {
                for binding in &file.imported_bindings {
                    if !binding.is_type_only {
                        if let Some(target) =
                            resolver
                                .resolve(&binding.specifier, path)
                                .and_then(|target| {
                                    edge_inputs
                                        .graph_files
                                        .visible_path(&target)
                                        .map(std::path::Path::to_path_buf)
                                })
                        {
                            output.modules.push((binding.specifier.clone(), target));
                        }
                    }
                }
            }
            if plan.calls {
                if let Some(replacements) = &file.value_flow.replacements {
                    output
                        .replacement_files
                        .push((path.clone(), std::sync::Arc::clone(replacements)));
                }
                value_flow::collect_constructors(&resolution, file, &mut output);
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
            all.modules.extend(part.modules);
            all.edges.extend(part.edges);
            all.sites.extend(part.sites);
            all.replacement_files.extend(part.replacement_files);
            all.classes.extend(part.classes);
            all.class_ids.extend(part.class_ids);
            all.constructor_bases.extend(part.constructor_bases);
            all.escapes.extend(part.escapes);
            all
        });
    for (module, file) in pass.modules.drain(..) {
        module_files.entry(module).or_default().insert(file);
    }
    if plan.calls {
        value_flow::prepare_constructor_routes(&mut pass);
        refine_value_calls(edge_inputs, facts, resolver, &indexes, &mut pass);
        if plan.effect_selectors {
            value_flow::resolve_value_exports(
                edge_inputs,
                facts,
                resolver,
                &indexes,
                &pass,
                callable_export_resolutions,
            );
        }
    }
    #[cfg(feature = "test-instrumentation")]
    value_flow_metrics::record_parameters(pass.parameter_cache.computations());
    #[cfg(feature = "test-instrumentation")]
    value_flow_metrics::record_nullish_work(pass.parameter_cache.nullish_work());
    #[cfg(feature = "test-instrumentation")]
    value_flow_metrics::record_origins(pass.parameter_cache.origin_nodes());
    #[cfg(feature = "test-instrumentation")]
    value_flow_metrics::record_execution_steps(pass.parameter_cache.execution_steps());
    #[cfg(feature = "test-instrumentation")]
    value_flow_metrics::record_dependency_links(pass.parameter_cache.dependency_links());
    if namespaces_in_repo {
        let mut escapes = NamespaceEscapes::default();
        for (file, root) in pass.escapes.drain(..) {
            escapes.entry(file).or_default().insert(root);
        }
        mark_escaped_namespaces(&mut pass.classes, &escapes);
    }
    pass
}
