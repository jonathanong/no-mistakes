struct CoreIndependentEdges {
    imports: Vec<Edge>,
    route_imports: Vec<Edge>,
    workspace: Vec<Edge>,
    package: Vec<Edge>,
    assets: Vec<Edge>,
    symbols: Vec<Edge>,
    tests: Vec<Edge>,
}

/// Import, route-import, workspace, package, asset, symbol, and test
/// collectors only read prepared inputs. Collect in parallel and merge on
/// the caller thread in the historical kind order.
fn collect_independent_core_edges(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    facts: Option<&dyn TsFactLookup>,
    resolver: &dyn ImportResolution,
    session: &crate::codebase::analysis_session::AnalysisSession,
    parsed_imports: &ParsedImports<'_>,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
) -> CoreIndependentEdges {
    let observer = session.observer().cloned();
    let ((imports, route_imports), ((workspace_edges, package), (assets, (symbols, tests)))) =
        rayon::join(
            || {
                crate::diagnostics::with_observer(observer.clone(), || {
                    rayon::join(
                        || {
                            traced_parallel_edges(observer.clone(), "graph.imports", || {
                                collect_import_edges_for_core(
                                    edge_inputs,
                                    parsed_imports,
                                    resolver,
                                    workspace,
                                )
                            })
                        },
                        || {
                            traced_parallel_edges(observer.clone(), "graph.route_imports", || {
                                collect_route_import_edges_for_core(edge_inputs, facts, session)
                            })
                        },
                    )
                })
            },
            || {
                crate::diagnostics::with_observer(observer.clone(), || {
                    rayon::join(
                        || {
                            crate::diagnostics::with_observer(observer.clone(), || {
                                rayon::join(
                                    || {
                                        traced_parallel_edges(
                                            observer.clone(),
                                            "graph.workspace",
                                            || {
                                                collect_workspace_edges_for_core(
                                                    edge_inputs,
                                                    parsed_imports,
                                                    resolver,
                                                    workspace,
                                                )
                                            },
                                        )
                                    },
                                    || {
                                        traced_parallel_edges(
                                            observer.clone(),
                                            "graph.package",
                                            || collect_package_edges_for_core(edge_inputs, workspace),
                                        )
                                    },
                                )
                            })
                        },
                        || {
                            crate::diagnostics::with_observer(observer.clone(), || {
                                rayon::join(
                                    || {
                                        traced_parallel_edges(
                                            observer.clone(),
                                            "graph.assets",
                                            || {
                                                collect_asset_edges_for_core(
                                                    edge_inputs,
                                                    parsed_imports,
                                                    resolver,
                                                )
                                            },
                                        )
                                    },
                                    || {
                                        crate::diagnostics::with_observer(observer.clone(), || {
                                            rayon::join(
                                                || {
                                                    traced_parallel_edges(
                                                        observer.clone(),
                                                        "graph.symbols",
                                                        || {
                                                            collect_symbol_edges_for_core(
                                                                edge_inputs,
                                                                facts,
                                                                resolver,
                                                                workspace,
                                                            )
                                                        },
                                                    )
                                                },
                                                || {
                                                    traced_parallel_edges(
                                                        observer.clone(),
                                                        "graph.tests",
                                                        || collect_test_edges_for_core(edge_inputs),
                                                    )
                                                },
                                            )
                                        })
                                    },
                                )
                            })
                        },
                    )
                })
            },
        );
    CoreIndependentEdges {
        imports,
        route_imports,
        workspace: workspace_edges,
        package,
        assets,
        symbols,
        tests,
    }
}

fn traced_parallel_edges(
    observer: Option<std::sync::Arc<crate::diagnostics::InvocationObserver>>,
    label: &'static str,
    collect: impl FnOnce() -> Vec<Edge> + Send,
) -> Vec<Edge> {
    with_observer_and_timing(observer, crate::diagnostics::TimingKind::Parallel, || {
        collect_unless_timed_out(|| crate::perf_trace::trace(label, collect))
    })
}

fn merge_independent_core_edges(forward: &mut EdgeMap, reverse: &mut EdgeMap, edges: CoreIndependentEdges) {
    merge_edges(forward, reverse, edges.imports);
    merge_edges(forward, reverse, edges.route_imports);
    merge_edges(forward, reverse, edges.workspace);
    merge_edges(forward, reverse, edges.package);
    merge_edges(forward, reverse, edges.assets);
    merge_edges(forward, reverse, edges.symbols);
    merge_edges(forward, reverse, edges.tests);
}
