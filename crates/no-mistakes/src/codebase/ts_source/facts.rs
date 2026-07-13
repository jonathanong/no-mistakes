use crate::codebase::dependencies::extract::{ExtractedImport, FunctionCall};
use crate::codebase::ts_http_calls::HttpCall;
use crate::codebase::ts_process_spawn::SpawnEdge;
use crate::codebase::ts_queues::usage::QueueUsage;
use crate::codebase::ts_routes::refs::{RouteHelper, RouteHelperImport, RouteHelperRef, RouteRef};
use crate::codebase::ts_symbols::FileSymbols;
use crate::queue::extract::FileFacts as QueueProjectFacts;
use crate::react_traits::report::types::ComponentFacts;
use crate::server_routes::model::FileFacts as ServerRouteFileFacts;
use std::collections::HashMap;
use std::path::PathBuf;

mod collect;
pub(crate) mod domain;
mod map;
mod plan;

pub use collect::{collect_ts_facts, collect_ts_facts_with_context};
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

#[derive(Debug, Clone, Default)]
pub struct TsFileFacts {
    /// Parser diagnostic for this source file. Facts may contain the parser's
    /// recovered AST, but consumers that require sound syntax can reject it.
    pub parse_error: Option<String>,
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

#[derive(Debug, Clone, Default)]
pub struct TsFactMap {
    facts: HashMap<PathBuf, TsFileFacts>,
    plan: TsFactPlan,
}

#[cfg(test)]
mod tests;
