use anyhow::Result;
use globset::{GlobBuilder, GlobSetBuilder};
use serde::Deserialize;
use std::path::{Path, PathBuf};

use super::workspace_resolution::{
    exports_to_entry_path, resolve_workspace_path, workspace_path_is_file,
};
pub use super::workspace_resolution::{IndexedWorkspaceMap, WorkspaceMap, WorkspacePackage};
use crate::codebase::ts_resolver::VisiblePathLookup;
use crate::codebase::{glob_normalize, ts_resolver::normalize_path};
#[cfg(not(test))]
macro_rules! workspace_load_metrics {
    ($root:expr) => {};
}
#[cfg(test)]
#[macro_use]
pub(crate) mod load_metrics_tests;
include!("workspaces/types.rs");
include!("workspaces/globs.rs");
include!("workspaces/pnpm_include.rs");
include!("workspaces/pnpm_include_load.rs");
include!("workspaces/pnpm_include_settings.rs");
include!("workspaces/pnpm_include_catalog.rs");
include!("workspaces/loading.rs");
include!("workspaces/package.rs");
pub(crate) mod edge_package_manifest;
mod manual;
pub(crate) use manual::{load_manual_workspace_from_source_store, ManualWorkspace};

#[cfg(test)]
mod tests;
