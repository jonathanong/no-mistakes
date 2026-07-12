use super::extract::{is_indexable, ExtractedImport, FunctionCall, ImportKind};
use crate::codebase::ts_resolver::{ImportResolver, TsConfig};
use crate::codebase::ts_source::facts::{
    collect_ts_facts, collect_ts_facts_with_context, TsFactContext, TsFactMap, TsFactPlan,
    TsFileFacts,
};
use crate::codebase::ts_symbols::ExportKind;
use crate::config::v2::{load_v2_config, ConfigView};
use anyhow::Result;
use globset::{Glob, GlobBuilder, GlobSet, GlobSetBuilder};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

include!("types.rs");
include!("build_plan.rs");
include!("graph_files.rs");
include!("files_config.rs");
include!("files_config_routes.rs");
include!("edge_maps.rs");

pub(crate) trait TsFactLookup: Sync {
    fn get_ts_facts(&self, path: &Path) -> Option<&TsFileFacts>;

    /// Already-collected Playwright test-file facts (URLs, selectors, text
    /// locators, helper references), when available. Lets a consumer skip
    /// re-parsing/re-analyzing a test file it already has facts for.
    /// `TsFactMap` never carries these (only `CheckFactMap` does), so the
    /// default returns `None` — callers must already tolerate a per-file
    /// `None` by falling back to full analysis for that file.
    fn get_playwright_facts(
        &self,
        _path: &Path,
    ) -> Option<&crate::codebase::check_facts::PlaywrightTestFacts> {
        None
    }
}

impl TsFactLookup for TsFactMap {
    fn get_ts_facts(&self, path: &Path) -> Option<&TsFileFacts> {
        self.get(path)
    }
}

impl TsFactLookup for crate::codebase::check_facts::CheckFactMap {
    fn get_ts_facts(&self, path: &Path) -> Option<&TsFileFacts> {
        self.ts.get(path).map(|facts| &facts.ts)
    }

    fn get_playwright_facts(
        &self,
        path: &Path,
    ) -> Option<&crate::codebase::check_facts::PlaywrightTestFacts> {
        self.ts
            .get(path)
            .and_then(|facts| facts.playwright.as_ref())
    }
}

include!("builder.rs");
include!("builder_helpers.rs");
include!("builder_entrypoints.rs");
include!("methods_lazy.rs");
include!("lazy_imports.rs");
include!("lazy_imports_owner_bridge.rs");
include!("lazy_import_symbols.rs");
include!("lazy_import_neighbors.rs");
include!("sort.rs");
include!("edge_import_reachability.rs");
include!("edge_imports.rs");
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
include!("edge_dotnet.rs");
include!("edge_swift.rs");
include!("edge_terraform.rs");
include!("filter.rs");
include!("symbol_index.rs");

#[cfg(test)]
pub(crate) mod test_support;

#[cfg(test)]
mod tests;
