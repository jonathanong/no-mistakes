/// Collect the domain-specific edge kinds that follow the core import, symbol,
/// workspace, and test relationships. Keeping this phase separate makes the
/// build order explicit without letting one orchestration file grow unbounded.
fn collect_remaining_edges(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    playwright_snapshot: Option<&crate::playwright::fsutil::VisiblePathSnapshot>,
    facts: Option<&dyn TsFactLookup>,
    resolution: EdgeResolutionContext<'_>,
    maps: EdgeMaps<'_>,
) -> Result<()> {
    let EdgeMaps {
        forward,
        reverse,
        resource_edge_details,
        resource_diagnostics,
    } = maps;
    let resolver = resolution.resolver;
    let root = edge_inputs.root;
    let plan = edge_inputs.plan;
    let graph_files = edge_inputs.graph_files;
    let config_options = edge_inputs.config_options;
    let default_ci = crate::config::v2::schema::CiConfig::default();
    let ci = config_options
        .map(|options| &options.ci)
        .unwrap_or(&default_ci);
    let parsed_workflows = (plan.ci || plan.workflow_topology)
        .then(|| parsed_workflows_for_graph(root, &graph_files.all, ci));

    crate::invocation::check_timeout()?;
    crate::perf_trace::trace("graph.markdown", || {
        if plan.markdown {
            merge_edges(
                forward,
                reverse,
                collect_md_edges(&graph_files.all, graph_files),
            );
        }
    });
    crate::invocation::check_timeout()?;
    crate::perf_trace::trace("graph.ci", || {
        if plan.ci {
            add_ci_edges(
                root,
                &graph_files.all,
                parsed_workflows
                    .as_ref()
                    .expect("CI graph plan prepares parsed workflows"),
                forward,
                reverse,
            );
        }
    });
    crate::invocation::check_timeout()?;
    crate::perf_trace::trace("graph.workflow_topology", || {
        if plan.workflow_topology {
            let parsed = parsed_workflows
                .as_ref()
                .expect("workflow topology graph plan prepares parsed workflows");
            let topology =
                crate::codebase::workflow_topology::load_workflow_topology_from_parsed(
                    root,
                    ci,
                    parsed,
                    &[],
                );
            merge_edges(
                forward,
                reverse,
                collect_workflow_topology_edges(root, &graph_files.all, ci, parsed, &topology),
            );
        }
    });
    crate::invocation::check_timeout()?;
    crate::perf_trace::trace("graph.routes", || {
        if plan.routes {
            let edges = collect_route_edges_with_graph_files(
                root,
                edge_inputs.tsconfig,
                edge_inputs.tsconfig_catalog,
                resolver,
                graph_files,
                facts,
                config_options,
            );
            merge_edges(forward, reverse, edges);
        }
    });
    crate::invocation::check_timeout()?;
    crate::perf_trace::trace("graph.queues", || {
        if plan.queues {
            let edges = collect_queue_edges(
                root,
                resolver,
                graph_files,
                facts,
                config_options,
            );
            merge_edges(forward, reverse, edges);
        }
    });
    crate::invocation::check_timeout()?;
    crate::perf_trace::trace("graph.playwright_routes", || -> Result<()> {
        if plan.playwright_routes {
            let Some(snapshot) = playwright_snapshot else {
                anyhow::bail!("Playwright graph plan requires a visible-path snapshot");
            };
            let edges = collect_playwright_route_edges_from_snapshot(
                root,
                edge_inputs.config_path,
                &graph_files.all,
                facts,
                snapshot,
                edge_inputs.playwright_settings,
            );
            merge_edges(forward, reverse, edges);
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
            let edges = collect_react_render_edges(root, facts, graph_files.indexable());
            merge_edges(forward, reverse, edges);
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

fn merge_resource_edge_details(into: &mut ResourceEdgeDetails, source: ResourceEdgeDetails) {
    for (edge, mut sites) in source {
        let entry = into.entry(edge).or_default();
        entry.append(&mut sites);
        entry.sort();
        entry.dedup();
    }
}
