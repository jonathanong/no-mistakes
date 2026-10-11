use super::super::configured_plan_candidates::CoverageHints;
use super::super::{SelectedTest, TestFramework, TestPlanGroupResult, Warning, WarningKey};
use super::lockfile_seeds::LockfileSeedResult;
use super::vitest_setup_groups::VitestSetupFallback;
use no_mistakes::codebase::dependencies::graph::DepGraph;
use no_mistakes::codebase::test_filter::TestFileFilter;
use no_mistakes::config::v2::schema::TestPlanGroup;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

pub(super) struct GroupSelectionInputs<'a, 'b> {
    pub(super) framework: TestFramework,
    pub(super) root: &'a Path,
    pub(super) changed_files: &'a [PathBuf],
    pub(super) graph: &'a DepGraph,
    pub(super) all_tests: &'a [PathBuf],
    pub(super) all_test_set: &'a HashSet<PathBuf>,
    pub(super) test_filter: &'a TestFileFilter,
    pub(super) coverage_hints: &'a CoverageHints,
    pub(super) groups: &'a [TestPlanGroup],
    pub(super) target_only_group_index: Option<usize>,
    pub(super) targeted_candidates: &'a [SelectedTest],
    pub(super) lockfile_seed_result: Option<&'a LockfileSeedResult>,
    pub(super) native_semantic_candidates: &'a [SelectedTest],
    pub(super) has_global_limit: bool,
    pub(super) remaining_global: usize,
    pub(super) warnings: Vec<Warning>,
    pub(super) warnings_seen: HashSet<WarningKey>,
    pub(super) vitest_setup_fallback: &'a mut VitestSetupFallback<'b>,
}

pub(super) struct GroupSelectionResult {
    pub(super) selected_map: BTreeMap<PathBuf, SelectedTest>,
    pub(super) used: HashSet<String>,
    pub(super) group_results: Vec<TestPlanGroupResult>,
    pub(super) remaining_global: usize,
    pub(super) warnings: Vec<Warning>,
    pub(super) warnings_seen: HashSet<WarningKey>,
    pub(super) fallback_reasons: Vec<String>,
    pub(super) lockfile_seeds_injected: bool,
}
