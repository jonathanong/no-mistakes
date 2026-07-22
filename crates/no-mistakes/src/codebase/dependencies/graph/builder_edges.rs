/// Collects every configured edge kind and merges each into `forward`/`reverse`,
/// one `crate::perf_trace::trace(...)`-wrapped block per kind. Split out of
/// `builder.rs`'s `build_with_plan_files_config_and_facts` purely to stay under
/// the 200-code-line-per-file cap after adding per-edge-kind timing — no
/// behavior change, this is the same sequence that used to live inline there.
struct EdgeMaps<'a> {
    forward: &'a mut EdgeMap,
    reverse: &'a mut EdgeMap,
    resource_edge_details: &'a mut ResourceEdgeDetails,
    resource_diagnostics: &'a mut Vec<ResourceGraphDiagnostic>,
}

struct EdgeResolutionContext<'a> {
    resolver: &'a dyn ImportResolution,
    session: &'a crate::codebase::analysis_session::AnalysisSession,
}

fn collect_and_merge_all_edges(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    playwright_snapshot: Option<&crate::playwright::fsutil::VisiblePathSnapshot>,
    facts: Option<&dyn TsFactLookup>,
    resolution: EdgeResolutionContext<'_>,
    parsed_imports: &ParsedImports<'_>,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    maps: EdgeMaps<'_>,
) -> Result<()> {
    let EdgeMaps {
        forward,
        reverse,
        resource_edge_details,
        resource_diagnostics,
    } = maps;
    let resolver = resolution.resolver;
    let session = resolution.session;
    let root = edge_inputs.root;
    let tsconfig = edge_inputs.tsconfig;
    let plan = edge_inputs.plan;
    let graph_files = edge_inputs.graph_files;
    let config_options = edge_inputs.config_options;
    let playwright_settings = edge_inputs.playwright_settings;
    let config_path = edge_inputs.config_path;
    let files = &graph_files.indexable;
    crate::invocation::check_timeout()?;
    crate::perf_trace::trace("graph.imports", || {
        if plan.imports {
            let import_edges =
                collect_import_edges(parsed_imports, resolver, workspace, graph_files);
            merge_edges(forward, reverse, import_edges);
        }
    });

    crate::invocation::check_timeout()?;
    crate::perf_trace::trace("graph.route_imports", || -> Result<()> {
        if plan.route_imports {
            let Some(facts) = facts else {
                anyhow::bail!("TS import facts are required for route-import edges");
            };
            let route_import_edges = collect_route_import_edges(
                files,
                facts,
                tsconfig,
                edge_inputs.tsconfig_catalog,
                graph_files,
                session,
            );
            merge_edges(forward, reverse, route_import_edges);
        }
        Ok(())
    })?;

    crate::invocation::check_timeout()?;
    crate::perf_trace::trace("graph.workspace", || {
        if plan.workspace {
            let workspace_edges =
                collect_workspace_edges(parsed_imports, resolver, workspace, graph_files);
            merge_edges(forward, reverse, workspace_edges);
        }
    });

    crate::invocation::check_timeout()?;
    crate::perf_trace::trace("graph.package", || {
        if plan.package {
            let workspace_manifest_edges =
                collect_workspace_manifest_edges(&graph_files.all, workspace, graph_files);
            merge_edges(forward, reverse, workspace_manifest_edges);
        }
    });

    crate::invocation::check_timeout()?;
    crate::perf_trace::trace("graph.assets", || {
        if plan.assets {
            merge_edges(
                forward,
                reverse,
                collect_asset_edges(parsed_imports, resolver, graph_files),
            );
        }
    });

    crate::invocation::check_timeout()?;
    crate::perf_trace::trace("graph.symbols", || -> Result<()> {
        if plan.symbols {
            let Some(facts) = facts else {
                anyhow::bail!("TS symbol facts are required when symbol edges are requested");
            };
            let symbol_edges = collect_symbol_edges(
                root,
                SymbolGraphFiles {
                    indexable: files,
                    all: &graph_files.all,
                    visible: graph_files.visible(),
                    graph_files,
                },
                facts,
                resolver,
                workspace,
                config_options,
            );
            merge_edges(forward, reverse, symbol_edges);
        }
        Ok(())
    })?;

    crate::invocation::check_timeout()?;
    crate::perf_trace::trace("graph.tests", || {
        if plan.tests {
            let test_edges = collect_test_edges(
                root,
                files,
                config_options.and_then(|options| options.test_filter.as_ref()),
            );
            merge_edges(forward, reverse, test_edges);
        }
    });

    crate::invocation::check_timeout()?;
    crate::perf_trace::trace("graph.markdown", || {
        if plan.markdown {
            let md_edges = collect_md_edges(&graph_files.all, graph_files);
            merge_edges(forward, reverse, md_edges);
        }
    });

    crate::invocation::check_timeout()?;
    crate::perf_trace::trace("graph.ci", || {
        if plan.ci {
            add_ci_edges(root, &graph_files.all, forward, reverse);
        }
    });

    crate::invocation::check_timeout()?;
    crate::perf_trace::trace("graph.routes", || {
        if plan.routes {
            let route_edges = collect_route_edges_with_graph_files(
                root,
                tsconfig,
                edge_inputs.tsconfig_catalog,
                resolver,
                graph_files,
                facts,
                config_options,
            );
            merge_edges(forward, reverse, route_edges);
        }
    });

    crate::invocation::check_timeout()?;
    crate::perf_trace::trace("graph.queues", || {
        if plan.queues {
            merge_edges(
                forward,
                reverse,
                collect_queue_edges(root, resolver, graph_files, facts, config_options),
            );
        }
    });

    crate::invocation::check_timeout()?;
    crate::perf_trace::trace("graph.playwright_routes", || -> Result<()> {
        if plan.playwright_routes {
            let Some(snapshot) = playwright_snapshot else {
                anyhow::bail!("Playwright graph plan requires a visible-path snapshot");
            };
            let playwright_edges = collect_playwright_route_edges_from_snapshot(
                root,
                config_path,
                &graph_files.all,
                facts,
                snapshot,
                playwright_settings,
            );
            merge_edges(forward, reverse, playwright_edges);
        }
        Ok(())
    })?;

    crate::invocation::check_timeout()?;
    crate::perf_trace::trace("graph.http_process", || {
        merge_http_process_edges(edge_inputs, facts, forward, reverse);
    });

    crate::invocation::check_timeout()?;
    crate::perf_trace::trace("graph.react", || {
        if plan.react {
            let react_edges = collect_react_render_edges(root, facts, graph_files.indexable());
            merge_edges(forward, reverse, react_edges);
        }
    });

    crate::invocation::check_timeout()?;
    crate::perf_trace::trace("graph.resources", || -> Result<()> {
        if plan.resources {
            let Some(facts) = facts else {
                anyhow::bail!("TS resource facts are required when resource edges are requested");
            };
            let (edges, details, diagnostics) = collect_resource_edges(
                root,
                graph_files.indexable(),
                facts,
                graph_files.resource_candidates(),
            );
            merge_edges(forward, reverse, edges);
            merge_resource_edge_details(resource_edge_details, details);
            resource_diagnostics.extend(diagnostics);
            resource_diagnostics.sort();
            resource_diagnostics.dedup();
        }
        Ok(())
    })?;

    crate::perf_trace::trace("graph.dotnet", || {
        merge_dotnet_edges(edge_inputs, forward, reverse);
    });
    crate::perf_trace::trace("graph.swift", || {
        merge_swift_edges(edge_inputs, facts, forward, reverse);
    });
    crate::perf_trace::trace("graph.terraform", || {
        merge_terraform_edges(edge_inputs, forward, reverse);
    });
    crate::invocation::check_timeout()?;
    Ok(())
}
