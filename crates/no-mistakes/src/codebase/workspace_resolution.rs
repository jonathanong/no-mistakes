use super::ts_resolver::{normalize_path, VisiblePathLookup};
use std::cmp::Ordering;
use std::path::{Path, PathBuf};

include!("workspace_resolution/models.rs");
include!("workspace_resolution/indexed.rs");
include!("workspace_resolution/projection_key.rs");
include!("workspace_resolution/package_model.rs");
include!("workspace_resolution/exports.rs");
include!("workspace_resolution/path.rs");

#[cfg(test)]
mod test_helpers;
