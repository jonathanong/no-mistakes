use anyhow::Result;
use globset::{GlobBuilder, GlobSetBuilder};
use serde::Deserialize;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};

use crate::codebase::ts_resolver::VisiblePathLookup;
use crate::codebase::{glob_normalize, ts_resolver::normalize_path};
include!("workspaces/models.rs");
include!("workspaces/indexed.rs");
include!("workspaces/package_model.rs");
include!("workspaces/types.rs");
include!("workspaces/globs.rs");
include!("workspaces/loading.rs");
include!("workspaces/package.rs");
#[cfg(not(test))]
macro_rules! record_preparation {
    ($kind:ident) => {};
}
#[cfg(test)]
#[macro_use]
pub(crate) mod preparation_work;
pub(crate) use record_preparation;
include!("workspaces/visible.rs");
include!("workspaces/exports.rs");
include!("workspaces/path.rs");

#[cfg(test)]
mod tests;
