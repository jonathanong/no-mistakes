use anyhow::{Context, Result};
use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

include!("ts_resolver/config_ast.rs");
include!("ts_resolver/config.rs");
include!("ts_resolver/resolve_entry.rs");
include!("ts_resolver/visible.rs");
include!("ts_resolver/resolver.rs");
include!("ts_resolver/resolver_impl.rs");
include!("ts_resolver/resolver_cache_impl.rs");
include!("ts_resolver/resolver_paths.rs");
include!("ts_resolver/resolver_candidates.rs");
include!("ts_resolver/path.rs");
include!("ts_resolver/catalog.rs");
include!("ts_resolver/catalog_config.rs");
include!("ts_resolver/scoped.rs");
include!("ts_resolver/scoped_setup.rs");
include!("ts_resolver/project_resolver.rs");

#[cfg(test)]
mod scoped_test_support;

#[cfg(test)]
#[path = "ts_resolver/tests/apply_own_errors.rs"]
mod catalog_apply_own_errors;
#[cfg(test)]
mod catalog_coverage_tests;
#[cfg(test)]
mod catalog_reference_tests;

#[cfg(test)]
mod tests;
