use crate::codebase::dependencies::graph::{
    ts_fact_plan_and_context_for_plan_with_config, DepGraph, EdgeKind, GraphBuildPlan, GraphFiles,
    NodeEntry, NodeId, TsFactLookup,
};
use crate::codebase::test_filter::TestFileFilter;
use crate::codebase::ts_source::relative_slash_path;
use crate::codebase::ts_symbols::{extract_symbols, ExportKind};
use crate::config::v2::load_v2_config;
use anyhow::{bail, Context};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::io::Write;

include!("impact_types.rs");
include!("impact_collect.rs");
include!("impact_collect_targets.rs");
include!("impact_collect_callers.rs");
include!("impact_collect_caller_helpers.rs");
include!("impact_collect_file_usage.rs");
include!("impact_collect_local_names.rs");
include!("impact_output.rs");
