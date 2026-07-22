use anyhow::{Context, Result};
use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

include!("ts_resolver/config.rs");
include!("ts_resolver/resolve_entry.rs");
include!("ts_resolver/resolver.rs");
include!("ts_resolver/resolver_impl.rs");
include!("ts_resolver/resolver_cache_impl.rs");
include!("ts_resolver/resolver_paths.rs");
include!("ts_resolver/path.rs");
include!("ts_resolver/catalog.rs");
include!("ts_resolver/catalog_config.rs");
include!("ts_resolver/scoped.rs");
include!("ts_resolver/scoped_setup.rs");

#[cfg(test)]
mod scoped_test_support;

#[cfg(test)]
mod tests;
