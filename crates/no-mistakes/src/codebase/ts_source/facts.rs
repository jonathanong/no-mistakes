use crate::codebase::dependencies::extract::{
    extract_import_facts_from_program_with_source, is_indexable, ExtractedImport, FunctionCall,
};
use crate::codebase::ts_http_calls::HttpCall;
use crate::codebase::ts_process_spawn::SpawnEdge;
use crate::codebase::ts_queues::usage::QueueUsage;
use crate::codebase::ts_routes::refs::{RouteHelper, RouteHelperImport, RouteHelperRef, RouteRef};
use crate::codebase::ts_symbols::{extract_symbols_from_program, FileSymbols};
use crate::queue::extract::FileFacts as QueueProjectFacts;
use crate::react_traits::report::types::ComponentFacts;
use crate::server_routes::model::FileFacts as ServerRouteFileFacts;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub(crate) mod domain;
pub use domain::{BackendRouteFact, TsFactContext};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TsFactPlan {
    pub imports: bool,
    pub function_calls: bool,
    pub symbols: bool,
    pub source: bool,
    pub route_refs: bool,
    pub backend_routes: bool,
    pub queue_usage: bool,
    pub queue_factory: bool,
    pub queue_project: bool,
    pub http_calls: bool,
    pub process_spawns: bool,
    pub server_routes: bool,
    pub react: bool,
}

impl TsFactPlan {
    pub fn imports() -> Self {
        Self {
            imports: true,
            function_calls: true,
            symbols: false,
            ..Self::default()
        }
    }

    pub fn imports_and_symbols() -> Self {
        Self {
            imports: true,
            function_calls: true,
            symbols: true,
            ..Self::default()
        }
    }

    pub fn is_empty(self) -> bool {
        !self.imports
            && !self.function_calls
            && !self.symbols
            && !self.source
            && !self.route_refs
            && !self.backend_routes
            && !self.queue_usage
            && !self.queue_factory
            && !self.queue_project
            && !self.http_calls
            && !self.process_spawns
            && !self.server_routes
            && !self.react
    }

    pub fn has_domain_facts(self) -> bool {
        self.route_refs
            || self.backend_routes
            || self.queue_usage
            || self.queue_factory
            || self.queue_project
            || self.http_calls
            || self.process_spawns
            || self.server_routes
    }

    pub fn covers(self, required: Self) -> bool {
        (!required.imports || self.imports)
            && (!required.function_calls || self.function_calls)
            && (!required.symbols || self.symbols)
            && (!required.source || self.source)
            && (!required.route_refs || self.route_refs)
            && (!required.backend_routes || self.backend_routes)
            && (!required.queue_usage || self.queue_usage)
            && (!required.queue_factory || self.queue_factory)
            && (!required.queue_project || self.queue_project)
            && (!required.http_calls || self.http_calls)
            && (!required.process_spawns || self.process_spawns)
            && (!required.server_routes || self.server_routes)
            && (!required.react || self.react)
    }
}

#[derive(Debug, Clone, Default)]
pub struct TsFileFacts {
    pub source: Option<String>,
    pub imports: Vec<ExtractedImport>,
    pub function_calls: Vec<FunctionCall>,
    pub symbol_references: Vec<FunctionCall>,
    pub exported_functions: Vec<String>,
    pub unknown_callers: Vec<Option<String>>,
    pub has_unknown_top_level_call: bool,
    pub symbols: Option<FileSymbols>,
    pub route_refs: Vec<RouteRef>,
    pub route_helpers: Vec<RouteHelper>,
    pub route_helper_imports: Vec<RouteHelperImport>,
    pub route_helper_refs: Vec<RouteHelperRef>,
    pub backend_routes: Vec<BackendRouteFact>,
    pub queue_usage: Option<QueueUsage>,
    pub queue_create_line: Option<u32>,
    pub queue_name: Option<String>,
    pub(crate) queue_project: Option<QueueProjectFacts>,
    pub http_calls: Vec<HttpCall>,
    pub process_spawns: Vec<SpawnEdge>,
    pub(crate) server_routes: Option<ServerRouteFileFacts>,
    pub react_components: Vec<ComponentFacts>,
}

pub type TsFactMap = HashMap<PathBuf, TsFileFacts>;

pub fn collect_ts_facts(files: &[PathBuf], plan: TsFactPlan) -> TsFactMap {
    assert!(
        !plan.has_domain_facts(),
        "domain fact plans require collect_ts_facts_with_context"
    );
    collect_ts_facts_with_context(files, plan, &TsFactContext::default())
}

#[cfg(test)]
thread_local! {
    /// Test-only call counter for [`collect_ts_facts_with_context`], incremented
    /// once per call (not per file). Thread-local so it only reflects the
    /// current test (the harness runs each `#[test]` fn on its own fresh OS
    /// thread). Used to prove a single invocation shares one parse pass across
    /// consumers (e.g. a `DepGraph` build and a caller-context lookup) instead
    /// of re-parsing the same files (see `crates/CLAUDE.md`: "assert on a call
    /// count, not value equality").
    pub(crate) static COLLECT_TS_FACTS_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub fn collect_ts_facts_with_context(
    files: &[PathBuf],
    plan: TsFactPlan,
    context: &TsFactContext,
) -> TsFactMap {
    #[cfg(test)]
    COLLECT_TS_FACTS_CALLS.with(|calls| calls.set(calls.get() + 1));

    files
        .par_iter()
        .filter(|path| is_indexable(path))
        .filter_map(|path| {
            collect_file_facts(path, plan, context).map(|facts| (path.clone(), facts))
        })
        .collect()
}

fn collect_file_facts(
    path: &Path,
    plan: TsFactPlan,
    context: &TsFactContext,
) -> Option<TsFileFacts> {
    let source = std::fs::read_to_string(path).ok()?;
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap_or_else(|_| SourceType::ts());
    let parsed = Parser::new(&allocator, &source, source_type).parse();
    let import_facts = if plan.imports || plan.function_calls {
        extract_import_facts_from_program_with_source(&parsed.program, &source)
    } else {
        Default::default()
    };
    let symbols = plan
        .symbols
        .then(|| extract_symbols_from_program(&parsed.program, &source));
    let domain = if plan.has_domain_facts() {
        domain::collect_domain_facts(&parsed.program, path, &source, plan, context)
    } else {
        domain::DomainFacts::default()
    };
    let react_components = if plan.react {
        crate::react_traits::analyze::file::analyze_program(
            path,
            &context.root,
            &source,
            &parsed.program,
        )
        .components
    } else {
        Vec::new()
    };
    Some(TsFileFacts {
        source: plan.source.then_some(source),
        imports: import_facts.imports,
        function_calls: import_facts.function_calls,
        symbol_references: import_facts.symbol_references,
        exported_functions: import_facts.exported_functions,
        unknown_callers: import_facts.unknown_callers,
        has_unknown_top_level_call: import_facts.has_unknown_top_level_call,
        symbols,
        route_refs: domain.route_refs,
        route_helpers: domain.route_helpers,
        route_helper_imports: domain.route_helper_imports,
        route_helper_refs: domain.route_helper_refs,
        backend_routes: domain.backend_routes,
        queue_usage: domain.queue_usage,
        queue_create_line: domain.queue_create_line,
        queue_name: domain.queue_name,
        queue_project: domain.queue_project,
        http_calls: domain.http_calls,
        process_spawns: domain.process_spawns,
        server_routes: domain.server_routes,
        react_components,
    })
}

#[cfg(test)]
mod tests;
