use anyhow::Result;
use globset::{Glob, GlobSetBuilder};
use serde::Deserialize;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::codebase::{glob_normalize, ts_resolver::normalize_path};
include!("workspaces/types.rs");
include!("workspaces/globs.rs");
include!("workspaces/package.rs");
include!("workspaces/path.rs");

#[cfg(test)]
mod tests;
