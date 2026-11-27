use super::call_sites::CallSiteFact;
use super::TsFactPlan;
use crate::codebase::ts_http_calls::HttpCall;
use crate::codebase::ts_process_spawn::{
    extract_spawn_edges_from_program, extract_spawn_edges_from_program_from_visible, SpawnEdge,
};
use crate::codebase::ts_routes::defs_backend::extract_backend_routes_from_program;
use crate::codebase::ts_routes::refs::{
    extract_route_ref_facts_from_program, RouteHelper, RouteHelperImport, RouteHelperRef, RouteRef,
};
use crate::queue_analysis_hooks::{self, QueueFileFacts};
use oxc_ast::ast::Program;
use std::path::Path;

#[path = "domain_types.rs"]
mod domain_types;
#[path = "domain_walk.rs"]
mod domain_walk;
#[path = "effect_calls.rs"]
mod effect_calls;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
#[path = "effect_calls_tests.rs"]
mod effect_calls_tests;
pub use domain_types::{BackendRouteFact, EffectCallFact, RscEnvironmentFact, TsFactContext};
pub(crate) use effect_calls::collect_effect_calls;

#[derive(Default)]
pub(crate) struct DomainFacts {
    pub route_refs: Vec<RouteRef>,
    pub route_helpers: Vec<RouteHelper>,
    pub route_helper_imports: Vec<RouteHelperImport>,
    pub route_helper_refs: Vec<RouteHelperRef>,
    pub backend_routes: Vec<BackendRouteFact>,
    pub queue: QueueFileFacts,
    pub http_calls: Vec<HttpCall>,
    pub process_spawns: Vec<SpawnEdge>,
    pub server_routes: Option<super::react_analysis::server::FileFacts>,
    pub rsc_environment: Option<RscEnvironmentFact>,
    pub trpc_procedures: Vec<String>,
    pub trpc_calls: Vec<crate::codebase::ts_trpc::TrpcCallFact>,
    pub call_sites: Vec<CallSiteFact>,
}

pub(crate) fn collect_domain_facts<'a>(
    program: &Program<'a>,
    path: &Path,
    source: &str,
    plan: TsFactPlan,
    context: &TsFactContext,
    (record_waits, scoping): (bool, Option<&oxc_semantic::Scoping>),
) -> DomainFacts {
    let route_file = route_file_name(path, context);
    let route_ref_facts = if plan.route_refs {
        extract_route_ref_facts_from_program(program, source, &route_file)
    } else {
        Default::default()
    };
    let mut backend_routes = Vec::new();
    if plan.backend_routes {
        for extractor in &context.backend_route_extractors {
            if !context.matches_glob(path, &extractor.glob) {
                continue;
            }
            for (route, line) in
                extract_backend_routes_from_program(program, source, &extractor.register_object)
            {
                backend_routes.push(BackendRouteFact {
                    register_object: extractor.register_object.clone(),
                    route,
                    line,
                });
            }
        }
    }
    let queue = if plan.queue_usage || plan.queue_factory || plan.queue_project {
        let collect = queue_analysis_hooks::hooks().collect;
        collect(
            program,
            path,
            source,
            plan,
            context,
            (record_waits, scoping),
        )
    } else {
        QueueFileFacts::default()
    };
    let http_prefixes: Vec<&str> = context.http_prefixes.iter().map(String::as_str).collect();
    let fused = domain_walk::collect_fused_domain_calls(program, source, plan, &http_prefixes);
    let process_spawns = if plan.process_spawns {
        match context.visible_files.as_deref() {
            Some(visible) => extract_spawn_edges_from_program_from_visible(
                program,
                source,
                path,
                &context.root,
                visible,
            ),
            None => extract_spawn_edges_from_program(program, source, path, &context.root),
        }
    } else {
        Vec::new()
    };
    let server_routes = (plan.server_routes && context.matches_server_route(path))
        .then(|| (super::react_analysis::hooks().extract_server_routes)(path, source, program));
    let rsc_environment = plan
        .rsc_environment
        .then(|| classify_rsc_environment(program));
    let trpc_procedures = if plan.trpc_router && context.matches_trpc_router(path) {
        crate::codebase::ts_trpc::extract_trpc_router_from_program(program).procedures
    } else {
        Vec::new()
    };
    DomainFacts {
        route_refs: route_ref_facts.route_refs,
        route_helpers: route_ref_facts.route_helpers,
        route_helper_imports: route_ref_facts.route_helper_imports,
        route_helper_refs: route_ref_facts.route_helper_refs,
        backend_routes,
        queue,
        http_calls: fused.http_calls,
        process_spawns,
        server_routes,
        rsc_environment,
        trpc_procedures,
        trpc_calls: fused.trpc_calls,
        call_sites: fused.call_sites,
    }
}

fn classify_rsc_environment(program: &Program<'_>) -> RscEnvironmentFact {
    let has_use_server = program
        .directives
        .iter()
        .any(|directive| directive.directive == "use server");
    let has_use_client = program
        .directives
        .iter()
        .any(|directive| directive.directive == "use client");
    match (has_use_server, has_use_client) {
        (true, _) => RscEnvironmentFact::Server,
        (_, true) => RscEnvironmentFact::Client,
        _ => RscEnvironmentFact::Unknown,
    }
}

fn route_file_name(path: &Path, context: &TsFactContext) -> String {
    path.strip_prefix(&context.root)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}
