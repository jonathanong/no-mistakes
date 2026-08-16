struct FactDomainRemainingEdges {
    routes: Vec<Edge>,
    queues: Vec<Edge>,
    http_process: Vec<Edge>,
    react: Vec<Edge>,
    resources: Option<(Vec<Edge>, ResourceEdgeDetails, Vec<ResourceGraphDiagnostic>)>,
}

/// Route, queue, HTTP/process, React, and resource collectors only read facts.
/// Collect their edges in parallel and merge on the caller thread.
fn collect_fact_domain_remaining_edges(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    facts: Option<&dyn TsFactLookup>,
    resolver: &dyn ImportResolution,
    session: &crate::codebase::analysis_session::AnalysisSession,
) -> Result<FactDomainRemainingEdges> {
    let observer = session.observer().cloned();
    let timing_kind = crate::diagnostics::current_timing_kind();
    let ((routes, queues), (http_process, (react, resources))) = rayon::join(
        || {
            crate::diagnostics::with_observer(observer.clone(), || {
                rayon::join(
                    || {
                        with_observer_and_timing(observer.clone(), timing_kind, || {
                            collect_unless_timed_out(|| {
                                crate::perf_trace::trace("graph.routes", || {
                                    collect_route_edges_for_plan(
                                        edge_inputs, facts, resolver, session,
                                    )
                                })
                            })
                        })
                    },
                    || {
                        with_observer_and_timing(observer.clone(), timing_kind, || {
                            collect_unless_timed_out(|| {
                                crate::perf_trace::trace("graph.queues", || {
                                    collect_queue_edges_for_plan(edge_inputs, facts, resolver)
                                })
                            })
                        })
                    },
                )
            })
        },
        || {
            crate::diagnostics::with_observer(observer.clone(), || {
                rayon::join(
                    || {
                        with_observer_and_timing(observer.clone(), timing_kind, || {
                            collect_unless_timed_out(|| {
                                crate::perf_trace::trace("graph.http_process", || {
                                    collect_http_process_edges(edge_inputs, facts)
                                })
                            })
                        })
                    },
                    || {
                        crate::diagnostics::with_observer(observer.clone(), || {
                            rayon::join(
                                || {
                                    with_observer_and_timing(observer.clone(), timing_kind, || {
                                        collect_unless_timed_out(|| {
                                            crate::perf_trace::trace("graph.react", || {
                                                collect_react_edges_for_plan(edge_inputs, facts)
                                            })
                                        })
                                    })
                                },
                                || {
                                    with_observer_and_timing(observer.clone(), timing_kind, || {
                                        collect_unless_timed_out_or(Ok(None), || {
                                            crate::perf_trace::trace("graph.resources", || {
                                                collect_resource_edges_for_plan(edge_inputs, facts)
                                            })
                                        })
                                    })
                                },
                            )
                        })
                    },
                )
            })
        },
    );
    Ok(FactDomainRemainingEdges {
        routes,
        queues,
        http_process,
        react,
        resources: resources?,
    })
}

fn merge_fact_domain_remaining_edges(
    forward: &mut EdgeMap,
    reverse: &mut EdgeMap,
    resource_edge_details: &mut ResourceEdgeDetails,
    resource_diagnostics: &mut Vec<ResourceGraphDiagnostic>,
    edges: FactDomainRemainingEdges,
) {
    merge_edges(forward, reverse, edges.routes);
    merge_edges(forward, reverse, edges.queues);
    merge_edges(forward, reverse, edges.http_process);
    merge_edges(forward, reverse, edges.react);
    if let Some((resource_edges, details, diagnostics)) = edges.resources {
        merge_edges(forward, reverse, resource_edges);
        merge_resource_edge_details(resource_edge_details, details);
        resource_diagnostics.extend(diagnostics);
        resource_diagnostics.sort();
        resource_diagnostics.dedup();
    }
}

fn collect_route_edges_for_plan(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    facts: Option<&dyn TsFactLookup>,
    resolver: &dyn ImportResolution,
    session: &crate::codebase::analysis_session::AnalysisSession,
) -> Vec<Edge> {
    if !edge_inputs.plan.routes {
        return Vec::new();
    }
    collect_route_edges_with_graph_files(
        edge_inputs.root,
        RouteGraphResolution {
            tsconfig: edge_inputs.tsconfig,
            tsconfig_catalog: edge_inputs.tsconfig_catalog,
            session,
        },
        resolver,
        edge_inputs.graph_files,
        facts,
        edge_inputs.config_options,
    )
}

fn collect_queue_edges_for_plan(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    facts: Option<&dyn TsFactLookup>,
    resolver: &dyn ImportResolution,
) -> Vec<Edge> {
    if !edge_inputs.plan.queues {
        return Vec::new();
    }
    collect_queue_edges(
        edge_inputs.root,
        resolver,
        edge_inputs.graph_files,
        facts,
        edge_inputs.config_options,
    )
}

fn collect_react_edges_for_plan(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    facts: Option<&dyn TsFactLookup>,
) -> Vec<Edge> {
    if !edge_inputs.plan.react {
        return Vec::new();
    }
    collect_react_render_edges(edge_inputs.root, facts, edge_inputs.graph_files.indexable())
}

type ResourceEdgeBatch = (Vec<Edge>, ResourceEdgeDetails, Vec<ResourceGraphDiagnostic>);

fn collect_resource_edges_for_plan(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    facts: Option<&dyn TsFactLookup>,
) -> Result<Option<ResourceEdgeBatch>> {
    if !edge_inputs.plan.resources {
        return Ok(None);
    }
    let Some(facts) = facts else {
        anyhow::bail!("TS resource facts are required when resource edges are requested");
    };
    Ok(Some(collect_resource_edges(
        edge_inputs.root,
        edge_inputs.graph_files.indexable(),
        facts,
        edge_inputs.graph_files.resource_candidates(),
    )))
}

fn merge_resource_edge_details(into: &mut ResourceEdgeDetails, source: ResourceEdgeDetails) {
    for (edge, mut sites) in source {
        let entry = into.entry(edge).or_default();
        entry.append(&mut sites);
        entry.sort();
        entry.dedup();
    }
}
