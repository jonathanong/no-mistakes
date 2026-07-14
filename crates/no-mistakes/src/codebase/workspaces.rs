use anyhow::Result;
use globset::{GlobBuilder, GlobSetBuilder};
use serde::Deserialize;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};

use crate::codebase::{glob_normalize, ts_resolver::normalize_path};
include!("workspaces/models.rs");
include!("workspaces/indexed.rs");
include!("workspaces/package_model.rs");
include!("workspaces/types.rs");
include!("workspaces/globs.rs");
include!("workspaces/package.rs");
include!("workspaces/exports.rs");
include!("workspaces/path.rs");

#[cfg(test)]
mod tests;
