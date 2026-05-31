use super::extract::{is_indexable, ExtractedImport, ImportKind};
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
include!("edge_maps.rs");

pub(crate) trait TsFactLookup: Sync {
    fn get_ts_facts(&self, path: &Path) -> Option<&TsFileFacts>;
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
}

include!("builder.rs");
include!("methods_lazy.rs");
include!("lazy_imports.rs");
include!("lazy_import_symbols.rs");
include!("lazy_import_neighbors.rs");
include!("sort.rs");
include!("edge_import_reachability.rs");
include!("edge_imports.rs");
include!("edge_symbols.rs");
include!("edge_symbols_exports.rs");
include!("edge_symbols_helpers.rs");
include!("edge_symbols_local_scopes.rs");
include!("edge_symbols_targets.rs");
include!("edge_symbols_fallbacks.rs");
include!("edge_symbols_scoped_imports.rs");
include!("edge_package_manifest.rs");
include!("edge_tests_md.rs");
include!("edge_ci.rs");
include!("edge_routes.rs");
include!("edge_route_defs.rs");
include!("edge_queues.rs");
include!("edge_queue_processors.rs");
include!("edge_playwright_routes.rs");
include!("edge_playwright_selectors.rs");
include!("edge_playwright_http_process.rs");
include!("edge_react.rs");
include!("filter.rs");
include!("symbol_index.rs");

#[cfg(test)]
pub(crate) mod test_support;

#[cfg(test)]
mod tests;
