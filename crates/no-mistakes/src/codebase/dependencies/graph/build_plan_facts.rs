fn allowed_requests_language_frontends(allowed: &HashSet<EdgeKind>) -> bool {
    [
        EdgeKind::PythonImport,
        EdgeKind::PythonReference,
        EdgeKind::GoImport,
        EdgeKind::GoReference,
        EdgeKind::RustUse,
        EdgeKind::RustMod,
        EdgeKind::RustPackage,
        EdgeKind::RubyRequire,
        EdgeKind::RubyReference,
        EdgeKind::PhpUse,
        EdgeKind::PhpPackage,
        EdgeKind::JavaImport,
        EdgeKind::JavaReference,
        EdgeKind::KotlinImport,
        EdgeKind::KotlinReference,
        EdgeKind::ElixirImport,
        EdgeKind::ElixirReference,
    ]
    .into_iter()
    .any(|kind| allowed.contains(&kind))
}

fn graph_plan_needs_config(plan: GraphBuildPlan) -> bool {
    plan.ci
        || plan.workflow_topology
        || plan.routes
        || plan.queues
        || plan.http
        || plan.tests
        || plan.dotnet
        || plan.swift
        || plan.terraform
        || plan.language_frontends
        || plan.trpc
}

fn effective_ts_fact_plan(
    plan: GraphBuildPlan,
    options: Option<&GraphConfigOptions>,
) -> TsFactPlan {
    let mut fact_plan = plan.ts_fact_plan();
    let route_refs_configured = options.is_some_and(route_ref_facts_configured);
    let route_backend_configured = options.is_some_and(route_backend_facts_configured);
    let http_configured = options.is_some_and(http_facts_configured);
    let queue_configured = options.is_some_and(queue_facts_configured);

    fact_plan.route_refs &= route_refs_configured;
    fact_plan.backend_routes &= route_backend_configured || http_configured;
    fact_plan.http_calls &= http_configured;
    fact_plan.symbols = plan.symbols || (fact_plan.symbols && queue_configured);
    fact_plan.queue_usage &= queue_configured;
    fact_plan.queue_factory &= queue_configured;
    fact_plan.queue_project &= queue_configured;
    let trpc_configured = options.is_some_and(trpc_facts_configured);
    fact_plan.trpc_router &= trpc_configured;
    fact_plan.trpc_calls &= trpc_configured;
    fact_plan.server_routes = options.is_some_and(|options| {
        options.project_route_globset.is_some() && (plan.routes || plan.swift)
    });
    fact_plan
}
