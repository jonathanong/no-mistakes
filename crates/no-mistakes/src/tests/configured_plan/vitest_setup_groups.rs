use crate::integration_tests::types::ConfigProject;
use crate::tests::{SelectedTest, TestFramework, TestPlanGroupResult};
use no_mistakes::codebase::test_discovery::DiscoveredTests;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

pub(super) struct VitestSetupFallbackInputs<'a> {
    pub(super) framework: TestFramework,
    pub(super) root: &'a Path,
    pub(super) changed_files: &'a [PathBuf],
    pub(super) deleted_files: &'a [PathBuf],
    pub(super) projects: Option<&'a [ConfigProject]>,
    pub(super) discovered: &'a DiscoveredTests,
    pub(super) has_global_limit: bool,
    pub(super) all_test_count: usize,
}

pub(super) struct VitestSetupSelection<'a> {
    pub(super) used: &'a mut HashSet<String>,
    pub(super) selected_map: &'a mut BTreeMap<PathBuf, SelectedTest>,
    pub(super) group_results: &'a mut Vec<TestPlanGroupResult>,
    pub(super) fallback_reasons: &'a mut Vec<String>,
}

pub(super) struct VitestSetupFallback<'a> {
    inputs: VitestSetupFallbackInputs<'a>,
    checked_dependency_group: bool,
}

impl<'a> VitestSetupFallback<'a> {
    pub(super) fn new(inputs: VitestSetupFallbackInputs<'a>) -> Self {
        Self {
            inputs,
            checked_dependency_group: false,
        }
    }

    pub(super) fn apply_dependency_group(
        &mut self,
        selection: VitestSetupSelection<'_>,
        result_index: usize,
        remaining_group: usize,
        remaining_global: usize,
    ) -> usize {
        self.checked_dependency_group = true;
        self.apply(
            selection,
            Some((result_index, remaining_group)),
            remaining_global,
        )
        .unwrap_or_default()
    }

    pub(super) fn checked_dependency_group(&self) -> bool {
        self.checked_dependency_group
    }

    pub(super) fn apply_without_dependency_group(
        &self,
        selection: VitestSetupSelection<'_>,
        remaining_global: usize,
    ) {
        self.apply(selection, None, remaining_global);
    }

    fn apply(
        &self,
        selection: VitestSetupSelection<'_>,
        dependency_group: Option<(usize, usize)>,
        remaining_global: usize,
    ) -> Option<usize> {
        let VitestSetupSelection {
            used,
            selected_map,
            group_results,
            fallback_reasons,
        } = selection;
        super::vitest_setup_fallback::apply_selection(
            self.inputs.framework,
            self.inputs.root,
            self.inputs.changed_files,
            self.inputs.deleted_files,
            self.inputs.projects,
            self.inputs.discovered,
            used,
            selected_map,
            group_results,
            fallback_reasons,
            dependency_group,
            remaining_global,
            self.inputs.has_global_limit,
            self.inputs.all_test_count,
        )
    }
}
