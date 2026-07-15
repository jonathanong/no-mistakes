fn graph_build_plan(options: &AnalyzeProjectOptions) -> Result<GraphBuildPlan> {
    let mut plan = GraphBuildPlan::default();
    for request in &options.reports {
        if super::graph_direction(&request.report_type).is_some() {
            let args = super::traverse_args(request, options)?;
            let allowed = relationship_filter(&args.relationships);
            plan.include(
                GraphBuildPlan::from_allowed(allowed.as_ref()).with_symbols(
                    crate::codebase::dependencies::traversal_needs_symbol_facts(&args),
                ),
            );
        } else if request.report_type == "importUsages" {
            plan.include(GraphBuildPlan {
                imports: true,
                ..Default::default()
            });
        } else if request.report_type == "symbols"
            && request.options.get("mode").and_then(Value::as_str) == Some("signature-impact")
        {
            plan.include(crate::codebase::symbols::signature_impact_graph_plan());
        } else if request.report_type == "flow" {
            let raw = super::flow_options(request, options)?;
            let parsed: crate::napi_api::options::FlowOptions = serde_json::from_str(&raw)?;
            let flow = crate::napi_api::project::build_flow_options(parsed)?;
            let allowed = relationship_filter(&flow.relationships);
            plan.include(GraphBuildPlan::from_allowed(allowed.as_ref()).with_symbols(true));
        } else if matches!(request.report_type.as_str(), "effects" | "rscCallers") {
            plan.include(runtime_import_graph_plan());
        }
    }
    Ok(plan)
}

fn framework_preparation_plan(
    options: &AnalyzeProjectOptions,
    graph_plan: GraphBuildPlan,
) -> Result<crate::codebase::test_discovery::FrameworkPreparationPlan> {
    let mut plan = crate::codebase::test_discovery::FrameworkPreparationPlan::for_graph(graph_plan);
    for request in &options.reports {
        if super::graph_direction(&request.report_type).is_some() {
            let args = super::traverse_args(request, options)?;
            plan.include_framework_names(args.tests.iter().map(String::as_str));
        }
    }
    Ok(plan)
}

fn check_fact_plan(
    options: &AnalyzeProjectOptions,
    traversal: &SharedTraversalContext,
) -> Result<crate::codebase::check_facts::CheckFactPlan> {
    let mut graph =
        crate::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan_with_prepared(
            traversal.root(),
            traversal.build_plan(),
            traversal.prepared_graph(),
        );
    let queue = options.reports.iter().any(|request| {
        matches!(
            request.report_type.as_str(),
            "queues" | "queueEdges" | "queueRelated" | "queueCheck"
        )
    });
    let react = options.reports.iter().any(|request| {
        matches!(
            request.report_type.as_str(),
            "reactAnalyze" | "reactCheck" | "reactUsages"
        )
    });
    let react_usages = options
        .reports
        .iter()
        .any(|request| request.report_type == "reactUsages");
    let symbols = options
        .reports
        .iter()
        .any(|request| request.report_type == "symbols");
    let signature = options.reports.iter().any(|request| {
        request.report_type == "symbols"
            && request.options.get("mode").and_then(Value::as_str) == Some("signature-impact")
    });
    if has_server_report(options) {
        graph.0.route_refs = true;
        graph.0.server_routes = true;
        crate::server_routes::configure_fact_context(
            &mut graph.1, traversal.root(), traversal.config(),
        );
    }
    for request in &options.reports {
        if request.report_type == "rscCallers" {
            graph.0.rsc_environment = true;
        } else if request.report_type == "effects" {
            let parsed = super::options::effects_options(request, options)?;
            let kind = parsed
                .kind
                .as_deref()
                .context("kind is required for effects")?;
            let selection = crate::effects_query::selection_from_config(
                traversal.config(),
                kind,
                &parsed.categories,
            )?;
            graph.0.effect_calls = true;
            graph.0.function_calls = true;
            for function in crate::effects_query::selection_fact_functions(&selection) {
                graph.1.effect_functions.insert(function, None);
            }
        }
    }
    Ok(crate::codebase::check_facts::CheckFactPlan {
        queue,
        queue_factory_names: traversal.config().queues.factories.clone(),
        react,
        react_usages,
        symbols: symbols || react_usages,
        source: signature,
        graph: graph.0,
        graph_context: graph.1,
        ..Default::default()
    })
}

fn runtime_import_graph_plan() -> GraphBuildPlan {
    let allowed = std::collections::HashSet::from([
        crate::codebase::dependencies::EdgeKind::Import,
        crate::codebase::dependencies::EdgeKind::DynamicImport,
        crate::codebase::dependencies::EdgeKind::Require,
        crate::codebase::dependencies::EdgeKind::WorkspaceImport,
    ]);
    GraphBuildPlan::from_allowed(Some(&allowed))
}
