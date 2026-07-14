use anyhow::{Context, Result};
use dashmap::DashMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

include!("ts_resolver/config.rs");
include!("ts_resolver/resolve_entry.rs");
include!("ts_resolver/resolver.rs");
include!("ts_resolver/resolver_impl.rs");
include!("ts_resolver/resolver_paths.rs");
include!("ts_resolver/path.rs");

#[cfg(test)]
mod tests;
