use anyhow::{Context, Result};
use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

include!("ts_resolver/config_ast.rs");
include!("ts_resolver/config.rs");
include!("ts_resolver/resolve_entry.rs");
include!("ts_resolver/visible_identity.rs");
include!("ts_resolver/visible.rs");
include!("ts_resolver/visible_forwarding.rs");
include!("ts_resolver/prepared_visible.rs");
include!("ts_resolver/session_trait.rs");
include!("ts_resolver/resolver.rs");
include!("ts_resolver/resolver_types.rs");
include!("ts_resolver/resolver_impl.rs");
include!("ts_resolver/resolver_builders.rs");
include!("ts_resolver/resolver_cache_impl.rs");
include!("ts_resolver/resolver_extensions.rs");
include!("ts_resolver/resolver_paths.rs");
include!("ts_resolver/resolver_candidates.rs");
include!("ts_resolver/path.rs");
include!("ts_resolver/catalog.rs");
include!("ts_resolver/catalog_config.rs");
include!("ts_resolver/scoped.rs");
include!("ts_resolver/scoped_policy.rs");
include!("ts_resolver/scoped_selection.rs");
include!("ts_resolver/scoped_setup.rs");
include!("ts_resolver/project_resolver.rs");

#[path = "ts_resolver/node_builtins.rs"]
mod node_builtins;
pub use node_builtins::is_bare_node_builtin;

#[doc(hidden)]
#[path = "ts_resolver/declarations/package.rs"]
pub mod declarations_package;

#[doc(hidden)]
#[path = "ts_resolver/declarations/type_config.rs"]
pub mod declarations_type_config;

#[cfg(any(test, feature = "test-instrumentation"))]
#[doc(hidden)]
pub mod catalog_membership_work;

#[cfg(any(test, feature = "test-instrumentation"))]
#[doc(hidden)]
pub mod scoped_test_support;

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
#[path = "ts_resolver/tests/apply_own_errors.rs"]
mod catalog_apply_own_errors;
#[cfg(any(test, feature = "test-instrumentation"))]
#[doc(hidden)]
#[path = "ts_resolver/tests/import_classification.rs"]
pub mod import_classification_test_support;

#[cfg(test)]
#[path = "ts_resolver/unit_tests.rs"]
mod unit_tests;

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
#[path = "ts_resolver/tests/parallel_catalog.rs"]
mod parallel_catalog_tests;
