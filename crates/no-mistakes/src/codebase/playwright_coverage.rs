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

/// Public wrapper around `collect_report_with_frontend_root` for use in the
/// dependency graph builder. Callers that already know the `frontend_root` can
/// skip the config-load step and go straight to route/visit collection.
pub(crate) fn collect_report_with_frontend_root_pub(
    root: &Path,
    frontend_root: &Path,
    all_files: &[PathBuf],
) -> Result<CoverageReport> {
    collect_report_with_frontend_root(
        root,
        frontend_root,
        crate::codebase::dependencies::test_globs("playwright"),
        all_files,
    )
}

#[cfg(test)]
mod tests;
