use anyhow::{bail, Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use rayon::prelude::*;
use serde::Serialize;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};

use crate::codebase::config::{load_config, Config, RouteOptions};
use crate::codebase::ts_routes::{defs_frontend, matcher};

include!("playwright_coverage/types.rs");
include!("playwright_coverage/run.rs");
include!("playwright_coverage/roots.rs");
include!("playwright_coverage/report.rs");
include!("playwright_coverage/output.rs");

#[cfg(test)]
mod tests;
