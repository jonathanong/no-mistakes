fn with_observer_and_timing<T>(
    observer: Option<std::sync::Arc<crate::diagnostics::InvocationObserver>>,
    kind: crate::diagnostics::TimingKind,
    operation: impl FnOnce() -> T,
) -> T {
    crate::diagnostics::with_observer(observer, || {
        crate::diagnostics::with_timing_kind(kind, operation)
    })
}

fn collect_unless_timed_out<T: Default>(collect: impl FnOnce() -> T) -> T {
    collect_unless_timed_out_or(T::default(), collect)
}

fn collect_unless_timed_out_or<T>(timed_out: T, collect: impl FnOnce() -> T) -> T {
    if crate::invocation::check_timeout().is_err() {
        timed_out
    } else {
        collect()
    }
}

/// Collect the domain-specific edge kinds that follow the core import, symbol,
/// workspace, and test relationships. Independent kinds collect `Vec<Edge>`
/// in parallel and merge on this thread so public graph output stays stable.
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
    let session = resolution.session;
    let root = edge_inputs.root;
    let plan = edge_inputs.plan;
    let graph_files = edge_inputs.graph_files;
    let config_options = edge_inputs.config_options;
    let default_ci = crate::config::v2::schema::CiConfig::default();
    let ci = config_options
        .map(|options| &options.ci)
        .unwrap_or(&default_ci);
    let parsed_workflows = (plan.ci || plan.workflow_topology).then(|| {
        parsed_workflows_for_graph(root, &graph_files.all, ci, edge_inputs.workflow_documents)
    });

    crate::invocation::check_timeout()?;
    let independent = collect_independent_remaining_edges(edge_inputs, facts, session);
    merge_independent_remaining_edges(forward, reverse, independent);

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
            let topology = crate::codebase::workflow_topology::load_workflow_topology_from_parsed(
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
    let fact_domain = collect_fact_domain_remaining_edges(edge_inputs, facts, resolver, session)?;
    merge_fact_domain_remaining_edges(
        forward,
        reverse,
        resource_edge_details,
        resource_diagnostics,
        fact_domain,
    );

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
    crate::perf_trace::trace("graph.language_frontends", || {
        if edge_inputs.plan.language_frontends
            || edge_inputs.plan.queues
            || edge_inputs.plan.routes
        {
            merge_language_frontend_edges(edge_inputs, forward, reverse);
        }
    });
    crate::invocation::check_timeout()?;
    Ok(())
}
