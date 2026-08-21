use crate::codebase::dependencies::extract::{ExtractedImport, FunctionCall};
use crate::codebase::ts_http_calls::HttpCall;
use crate::codebase::ts_process_spawn::SpawnEdge;
use crate::codebase::ts_queues::usage::QueueUsage;
use crate::codebase::ts_resources::{ResourceCall, ResourceDiagnostic};
use crate::codebase::ts_routes::refs::{RouteHelper, RouteHelperImport, RouteHelperRef, RouteRef};
use crate::codebase::ts_symbols::FileSymbols;
use crate::queue::extract::FileFacts as QueueProjectFacts;
use crate::react_traits::report::types::ComponentFacts;
use crate::server_routes::model::FileFacts as ServerRouteFileFacts;
use std::sync::Arc;

pub(crate) mod call_sites;
mod collect;
pub(crate) mod domain;
mod map;
mod map_iter;
mod plan;

pub use call_sites::CallSiteFact;
pub(crate) use collect::{
    collect_file_facts_from_program, collect_ts_facts_with_context_sources_and_session,
    collect_ts_facts_with_context_sources_and_session_serializing_paths,
};
pub use collect::{
    collect_ts_facts, collect_ts_facts_with_context, collect_ts_facts_with_context_and_sources,
    collect_ts_facts_with_session_and_context,
};
pub use domain::{BackendRouteFact, EffectCallFact, RscEnvironmentFact, TsFactContext};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TsFactPlan {
    pub imports: bool,
    pub function_calls: bool,
    pub call_sites: bool,
    pub resources: bool,
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
    pub effect_calls: bool,
    pub rsc_environment: bool,
}

#[derive(Debug, Clone, Default)]
pub struct TsFileFacts {
    /// A source read or collector failure. Consumers that need a complete
    /// answer must surface this instead of treating the empty facts as a
    /// successful analysis result.
    pub operational_error: Option<String>,
    /// Parser diagnostic for this source file. Facts may contain the parser's
    /// recovered AST, but consumers that require sound syntax can reject it.
    pub parse_error: Option<String>,
    /// Whether [`Self::parse_error`] came from a parser panic rather than a
    /// recoverable diagnostic. Such partial facts must not answer sound
    /// symbol queries.
    pub fatal_parse_error: bool,
    pub source: Option<std::sync::Arc<str>>,
    pub imports: Vec<ExtractedImport>,
    pub function_calls: Vec<FunctionCall>,
    pub call_sites: Vec<CallSiteFact>,
    pub resource_calls: Vec<ResourceCall>,
    pub resource_diagnostics: Vec<ResourceDiagnostic>,
    pub symbol_references: Vec<FunctionCall>,
    pub exported_functions: Vec<String>,
    pub exported_resource_roots: Vec<String>,
    pub exported_resource_scopes: Vec<String>,
    pub unknown_callers: Vec<Option<String>>,
    pub has_unknown_top_level_call: bool,
    pub symbols: Option<Arc<FileSymbols>>,
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
    pub effect_calls: Vec<EffectCallFact>,
    pub rsc_environment: Option<RscEnvironmentFact>,
}

#[derive(Clone)]
enum TsFactSlot {
    Owned(Box<TsFileFacts>),
    Shared(Arc<TsFileFacts>),
}

impl TsFactSlot {
    fn as_facts(&self) -> &TsFileFacts {
        match self {
            Self::Owned(facts) => facts,
            Self::Shared(facts) => facts.as_ref(),
        }
    }

    fn into_owned(self) -> TsFileFacts {
        match self {
            Self::Owned(facts) => *facts,
            Self::Shared(facts) => Arc::unwrap_or_clone(facts),
        }
    }
}

#[derive(Clone, Default)]
pub struct TsFactMap {
    facts: crate::codebase::ts_source::FileIdMap<TsFactSlot>,
    plan: TsFactPlan,
}

impl std::fmt::Debug for TsFactMap {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_map().entries(self.iter()).finish()
    }
}

#[cfg(test)]
mod tests;
