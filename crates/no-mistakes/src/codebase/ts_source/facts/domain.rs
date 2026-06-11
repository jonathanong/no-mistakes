use super::TsFactPlan;
use crate::codebase::ts_http_calls::{extract_http_calls_from_program, HttpCall};
use crate::codebase::ts_process_spawn::{extract_spawn_edges_from_program, SpawnEdge};
use crate::codebase::ts_queues::factory::{
    find_create_queue_line_from_program, find_queue_name_from_program,
};
use crate::codebase::ts_queues::usage::{extract_queue_usage_from_program, QueueUsage};
use crate::codebase::ts_routes::defs_backend::extract_backend_routes_from_program;
use crate::codebase::ts_routes::refs::{
    extract_route_ref_facts_from_program, RouteHelper, RouteHelperImport, RouteHelperRef, RouteRef,
};
use oxc_ast::ast::Program;
use std::path::Path;

#[path = "domain_types.rs"]
mod domain_types;
pub use domain_types::{BackendRouteFact, TsFactContext};

#[derive(Default)]
pub(crate) struct DomainFacts {
    pub route_refs: Vec<RouteRef>,
    pub route_helpers: Vec<RouteHelper>,
    pub route_helper_imports: Vec<RouteHelperImport>,
    pub route_helper_refs: Vec<RouteHelperRef>,
    pub backend_routes: Vec<BackendRouteFact>,
    pub queue_usage: Option<QueueUsage>,
    pub queue_create_line: Option<u32>,
    pub queue_name: Option<String>,
    pub queue_project: Option<crate::queue::extract::FileFacts>,
    pub http_calls: Vec<HttpCall>,
    pub process_spawns: Vec<SpawnEdge>,
    pub server_routes: Option<crate::server_routes::model::FileFacts>,
}

pub(crate) fn collect_domain_facts<'a>(
    program: &Program<'a>,
    path: &Path,
    source: &str,
    plan: TsFactPlan,
    context: &TsFactContext,
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
    let queue_usage = plan
        .queue_usage
        .then(|| extract_queue_usage_from_program(program, source));
    let (queue_create_line, queue_name) = queue_factory_facts(program, path, source, plan, context);
    let queue_project = plan.queue_project.then(|| {
        crate::queue::extract::extract_program_with_factories(
            path,
            source,
            program,
            &context.queue_project_factory_names,
        )
    });
    let http_prefixes: Vec<&str> = context.http_prefixes.iter().map(String::as_str).collect();
    let http_calls = if plan.http_calls {
        extract_http_calls_from_program(program, source, &http_prefixes)
    } else {
        Vec::new()
    };
    let process_spawns = if plan.process_spawns {
        extract_spawn_edges_from_program(program, source, path, &context.root)
    } else {
        Vec::new()
    };
    let server_routes = plan
        .server_routes
        .then(|| crate::server_routes::extract::extract_program(path, source, program));
    DomainFacts {
        route_refs: route_ref_facts.route_refs,
        route_helpers: route_ref_facts.route_helpers,
        route_helper_imports: route_ref_facts.route_helper_imports,
        route_helper_refs: route_ref_facts.route_helper_refs,
        backend_routes,
        queue_usage,
        queue_create_line,
        queue_name,
        queue_project,
        http_calls,
        process_spawns,
        server_routes,
    }
}

fn queue_factory_facts<'a>(
    program: &Program<'a>,
    path: &Path,
    source: &str,
    plan: TsFactPlan,
    context: &TsFactContext,
) -> (Option<u32>, Option<String>) {
    if !plan.queue_factory || !context.matches_queue_factory(path) {
        return (None, None);
    }
    match (
        context.queue_factory_specifier.as_deref(),
        context.queue_factory_function.as_deref(),
    ) {
        (Some(factory_specifier), Some(factory_function)) => (
            find_create_queue_line_from_program(
                program,
                source,
                factory_specifier,
                factory_function,
            ),
            find_queue_name_from_program(program, factory_specifier, factory_function),
        ),
        _ => (None, None),
    }
}

fn route_file_name(path: &Path, context: &TsFactContext) -> String {
    path.strip_prefix(&context.root)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}
