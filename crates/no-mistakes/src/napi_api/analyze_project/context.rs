use super::options::{playwright_options, project_options};
use super::types::{AnalyzeProjectOptions, AnalyzeReportRequest};
use crate::codebase::dependencies::graph::GraphBuildPlan;
use crate::codebase::dependencies::{relationship_filter, Direction, SharedTraversalContext};
use crate::napi_api::options::{PlaywrightOptions, ProjectOptions};
use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

include!("context/check_prepare.rs");
include!("context/check_run.rs");
include!("context/scope_types.rs");
include!("context/scope_prepare.rs");
include!("context/scope_materialize.rs");
include!("context/traversal_report_keys.rs");
include!("context/scope_graph_reports.rs");
include!("context/scope_project_reports.rs");
include!("context/scope_cached_reports.rs");
include!("context/api.rs");
include!("context/api_reports.rs");
include!("context/scope_helpers.rs");
include!("context/plan_helpers.rs");
include!("context/target_helpers.rs");
include!("context/playwright_helpers.rs");
include!("context_render.rs");

#[cfg(test)]
mod api_tests;
#[cfg(test)]
mod scope_helpers_tests;
#[cfg(test)]
mod tests;
