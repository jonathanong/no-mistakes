use super::extract::{is_indexable, ExtractedImport, FunctionCall, ImportKind};
use crate::codebase::ts_resolver::{ImportResolution, ImportResolver, TsConfig};
use crate::codebase::ts_source::facts::{
    collect_ts_facts, collect_ts_facts_with_session_and_context, TsFactContext, TsFactMap,
    TsFactPlan, TsFileFacts,
};
use crate::codebase::ts_symbols::ExportKind;
use crate::config::v2::{load_v2_config, ConfigView};
use anyhow::Result;
use globset::{Glob, GlobBuilder, GlobSet, GlobSetBuilder};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::edge_index::{CanonicalEdge, EdgeIndex};

include!("types.rs");
include!("build_plan.rs");
include!("graph_files.rs");
include!("files_config.rs");
include!("files_config_prepared.rs");
include!("files_config_fact_context.rs");
include!("files_config_routes.rs");
include!("edge_maps.rs");
include!("fact_lookup.rs");

include!("builder.rs");
include!("builder_check_facts.rs");
include!("builder_observability.rs");
include!("builder_parse_errors.rs");
include!("builder_core_resolution.rs");
include!("builder_core.rs");
include!("builder_edges.rs");
include!("builder_remaining_edges.rs");
include!("builder_helpers.rs");
include!("builder_entrypoints.rs");
include!("methods_lazy.rs");
include!("lazy_import_types.rs");
include!("lazy_import_entrypoints.rs");
include!("lazy_imports.rs");
include!("lazy_imports_owner_bridge.rs");
include!("lazy_import_symbols.rs");
include!("lazy_import_neighbors.rs");
include!("bfs.rs");
include!("sort.rs");
include!("edge_import_reachability_scopes.rs");
include!("edge_import_reachability.rs");
include!("edge_imports.rs");
include!("edge_route_imports.rs");
include!("edge_symbols_types.rs");
include!("edge_symbols.rs");
include!("edge_symbols_call_graph.rs");
include!("edge_symbols_http.rs");
include!("edge_symbols_runtime.rs");
include!("edge_symbols_exports.rs");
include!("edge_symbols_star_candidates.rs");
include!("edge_symbols_star_reexports.rs");
include!("edge_symbols_star_shadow_keys.rs");
include!("edge_symbols_helpers.rs");
include!("edge_symbols_reexport_namespaces.rs");
include!("edge_symbols_local_scopes.rs");
include!("edge_symbols_targets.rs");
include!("edge_symbols_fallbacks.rs");
include!("edge_symbols_scoped_imports.rs");
include!("edge_package_manifest.rs");
include!("edge_tests_md.rs");
include!("edge_ci.rs");
include!("edge_workflow_commands.rs");
include!("edge_workflow_run.rs");
include!("edge_workflow_uses.rs");
include!("edge_workflow_topology.rs");
include!("edge_routes.rs");
include!("edge_route_helper_refs.rs");
include!("edge_route_helper_ref_wrappers.rs");
include!("edge_route_defs.rs");
include!("edge_queues.rs");
include!("edge_queue_processors.rs");
include!("edge_playwright_routes.rs");
include!("edge_playwright_selectors.rs");
include!("edge_playwright_http_process.rs");
include!("edge_react.rs");
include!("edge_resources.rs");
include!("edge_resource_resolution.rs");
#[cfg(test)]
mod edge_resources_tests;
include!("edge_dotnet.rs");
include!("edge_swift.rs");
include!("edge_terraform.rs");
include!("filter.rs");
include!("symbol_index.rs");

#[cfg(test)]
pub(crate) mod test_support;

#[cfg(test)]
mod tests;
