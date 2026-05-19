use super::TsFactPlan;
use crate::codebase::ts_http_calls::{extract_http_calls_from_program, HttpCall};
use crate::codebase::ts_process_spawn::{extract_spawn_edges_from_program, SpawnEdge};
use crate::codebase::ts_queues::factory::{
    find_create_queue_line_from_program, find_queue_name_from_program,
};
use crate::codebase::ts_queues::usage::{extract_queue_usage_from_program, QueueUsage};
use crate::codebase::ts_routes::defs_backend::extract_backend_routes_from_program;
use crate::codebase::ts_routes::refs::{extract_route_refs_from_program, RouteRef};
use oxc_ast::ast::Program;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct TsFactContext {
    pub root: PathBuf,
    pub backend_register_object: Option<String>,
    pub queue_factory_specifier: Option<String>,
    pub queue_factory_function: Option<String>,
    pub http_prefixes: Vec<String>,
}

impl TsFactContext {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            ..Self::default()
        }
    }
}

impl Default for TsFactContext {
    fn default() -> Self {
        Self {
            root: PathBuf::new(),
            backend_register_object: None,
            queue_factory_specifier: None,
            queue_factory_function: None,
            http_prefixes: Vec::new(),
        }
    }
}

pub(crate) struct DomainFacts {
    pub route_refs: Vec<RouteRef>,
    pub backend_routes: Vec<(String, u32)>,
    pub queue_usage: Option<QueueUsage>,
    pub queue_create_line: Option<u32>,
    pub queue_name: Option<String>,
    pub http_calls: Vec<HttpCall>,
    pub process_spawns: Vec<SpawnEdge>,
}

pub(crate) fn collect_domain_facts<'a>(
    program: &Program<'a>,
    path: &Path,
    source: &str,
    plan: TsFactPlan,
    context: &TsFactContext,
) -> DomainFacts {
    let route_file = route_file_name(path, context);
    let route_refs = if plan.route_refs {
        extract_route_refs_from_program(program, source, &route_file)
    } else {
        Vec::new()
    };
    let backend_routes = context
        .backend_register_object
        .as_ref()
        .filter(|_| plan.backend_routes)
        .map(|register_object| {
            extract_backend_routes_from_program(program, source, register_object)
        })
        .unwrap_or_default();
    let queue_usage = plan
        .queue_usage
        .then(|| extract_queue_usage_from_program(program, source));
    let (queue_create_line, queue_name) = queue_factory_facts(program, source, plan, context);
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
    DomainFacts {
        route_refs,
        backend_routes,
        queue_usage,
        queue_create_line,
        queue_name,
        http_calls,
        process_spawns,
    }
}

fn queue_factory_facts<'a>(
    program: &Program<'a>,
    source: &str,
    plan: TsFactPlan,
    context: &TsFactContext,
) -> (Option<u32>, Option<String>) {
    if !plan.queue_factory {
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
