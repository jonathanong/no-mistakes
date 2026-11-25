use super::configured_plan_candidates::merge_selected;
use super::diff_parser::DiffFile;
use super::plan::relative_path;
use super::{
    push_resource_diagnostics, warning_key, PlanArgs, SelectedTest, TestFramework, TestPlan,
    TestPlanGroupResult, Warning, WarningKey,
};
use anyhow::Result;
use no_mistakes::codebase::test_discovery::DiscoveredTests;
use no_mistakes::codebase::workspaces::WorkspaceMap;
use no_mistakes::config::v2::schema::NoMistakesConfig;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

mod coverage_support;
pub(super) mod dep_triggers;
mod direct_test_owner;
mod discovery;
mod environment;
mod fallback;
mod finalize;
mod graph;
mod group_selection;
mod group_selection_types;
mod hints;
mod hints_domains;
pub(crate) mod lockfile_seeds;
mod native_fallback;
mod native_fallback_lang;
mod native_semantic_fallback;
pub(crate) mod native_semantic_seeds;
mod targeted_triggers;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
pub(super) mod vitest_setup_fallback;
mod vitest_setup_groups;
use dep_triggers::dependency_triggers;
pub(crate) use direct_test_owner::generate_direct_test_owner_plan_with_prepared;
pub(crate) use discovery::discover_framework_tests_from_prepared;
use environment::{
    configured_environment, configured_groups, effective_global_config_fallback, framework_name,
    limit_count, override_limit,
};
use fallback::{fallback_plan, FallbackRequest};
use finalize::{attach_targets, sorted_selected_tests, sorted_warnings};
use group_selection::select_configured_groups;
use group_selection_types::GroupSelectionInputs;
use hints::build_coverage_hints_from_prepared;
use lockfile_seeds::{apply_lockfile_seeds, lockfile_seed_candidates};
use native_fallback::{native_fallback_selection, native_traceable_changed_files};
use native_semantic_fallback::native_semantic_fallback_plan;
use native_semantic_seeds::native_semantic_seed_candidates;
use targeted_triggers::{insert_synthesized_dependency_group, targeted_dependency_candidates};
use vitest_setup_groups::{VitestSetupFallback, VitestSetupFallbackInputs, VitestSetupSelection};

include!("configured_plan/generate.rs");
include!("configured_plan/generate_tail.rs");
include!("configured_plan/early_fallback.rs");
