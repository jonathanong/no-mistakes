use crate::codebase::dependencies::graph::{
    DepGraph, EdgeKind, GraphBuildPlan, GraphFiles, NodeEntry, NodeId,
};
use crate::codebase::test_filter::TestFileFilter;
use crate::codebase::ts_source::facts::TsFactMap;
use crate::codebase::ts_source::relative_slash_path;
use crate::codebase::ts_symbols::ExportKind;
use anyhow::{bail, Context};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::io::Write;

include!("impact_types.rs");
include!("impact_collect.rs");
include!("impact_collect_targets.rs");
include!("impact_collect_callers.rs");
#[path = "impact_collect_caller_helpers.rs"]
mod impact_collect_caller_helpers;
use impact_collect_caller_helpers::{
    caller_is_target_export, is_test_like_file, legacy_call_matches_local_target,
    matches_local_callee,
};
include!("impact_collect_file_usage.rs");
include!("impact_collect_local_names.rs");
include!("impact_output.rs");

#[cfg(test)]
#[path = "impact/tests/caller_helper_coverage.rs"]
mod impact_caller_helper_coverage;
#[cfg(test)]
mod impact_collect_targets_tests;
#[cfg(test)]
mod impact_test_support;
#[cfg(test)]
#[path = "impact/tests/write_errors.rs"]
mod impact_write_errors;
