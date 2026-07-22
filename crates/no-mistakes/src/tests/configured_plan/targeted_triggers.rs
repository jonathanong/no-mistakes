use super::super::configured_plan_candidates::merge_selected;
use super::super::plan::relative_path;
use super::super::{Confidence, ImpactReason, SelectedTest};
use super::TestFramework;
use no_mistakes::codebase::test_discovery::DiscoveredTests;
use no_mistakes::config::v2::schema::TestPlanGroupType;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};

pub(super) struct TargetedOverlapRecovery {
    test_files: HashSet<String>,
    dependencies_have_run: bool,
}

impl TargetedOverlapRecovery {
    pub(super) fn new(targeted: &[SelectedTest]) -> Self {
        Self {
            test_files: targeted
                .iter()
                .map(|candidate| candidate.test_file.clone())
                .collect(),
            dependencies_have_run: false,
        }
    }

    pub(super) fn should_recover(
        &self,
        framework: TestFramework,
        group: TestPlanGroupType,
    ) -> bool {
        self.dependencies_have_run
            && (group == TestPlanGroupType::Direct
                || (framework == TestFramework::Playwright && group == TestPlanGroupType::Coverage))
    }

    pub(super) fn candidate_used_override(
        &self,
        used: &HashSet<String>,
        recover: bool,
    ) -> Option<HashSet<String>> {
        if !recover {
            return None;
        }
        Some(
            used.iter()
                .filter(|test_file| !self.test_files.contains(*test_file))
                .cloned()
                .collect(),
        )
    }

    pub(super) fn merge_existing(
        &self,
        root: &Path,
        candidates: &mut Vec<SelectedTest>,
        used: &HashSet<String>,
        selected: &mut BTreeMap<PathBuf, SelectedTest>,
    ) {
        for candidate in candidates
            .iter()
            .filter(|candidate| used.contains(&candidate.test_file))
        {
            if let Some(existing) = selected.get_mut(&root.join(&candidate.test_file)) {
                merge_selected(existing, candidate);
                // An independent later group owns this test too, so keep
                // every discovered runner target when finalizing it.
                existing.targets.clear();
            }
        }
        candidates.retain(|candidate| !used.contains(&candidate.test_file));
    }

    pub(super) fn finish_group(&mut self, group: TestPlanGroupType) {
        self.dependencies_have_run |= group == TestPlanGroupType::Dependencies;
    }
}

pub(super) fn targeted_dependency_candidates(
    root: &Path,
    all_tests: &[PathBuf],
    discovered: &DiscoveredTests,
    changed_targets: &BTreeMap<PathBuf, BTreeSet<String>>,
) -> Vec<SelectedTest> {
    let requested_targets = changed_targets
        .values()
        .flat_map(|targets| targets.iter().cloned())
        .collect::<BTreeSet<_>>();
    let mut selected = Vec::new();
    for test_path in all_tests {
        let selected_targets = discovered
            .targets_by_path
            .get(test_path)
            .into_iter()
            .flatten()
            .filter(|target| {
                target
                    .project
                    .as_ref()
                    .is_some_and(|project| requested_targets.contains(project))
            })
            .cloned()
            .collect::<Vec<_>>();
        if selected_targets.is_empty() {
            continue;
        }
        let target_names = selected_targets
            .iter()
            .filter_map(|target| target.project.as_ref())
            .collect::<BTreeSet<_>>();
        let reasons = changed_targets
            .iter()
            .filter(|(_, targets)| targets.iter().any(|target| target_names.contains(&target)))
            .map(|(changed, _)| {
                let changed_file = relative_path(root, changed);
                ImpactReason {
                    changed_file: changed_file.clone(),
                    path: vec![changed_file, relative_path(root, test_path)],
                    via: vec!["configured-trigger".to_string()],
                }
            })
            .collect();
        selected.push(SelectedTest {
            test_file: relative_path(root, test_path),
            confidence: Confidence::High,
            reasons,
            targets: selected_targets,
        });
    }
    selected
}

pub(super) fn merge_targeted_candidates(
    root: &Path,
    candidates: &mut Vec<SelectedTest>,
    targeted: &[SelectedTest],
    used: &HashSet<String>,
    selected_map: &mut BTreeMap<PathBuf, SelectedTest>,
) {
    for targeted_test in targeted {
        if used.contains(&targeted_test.test_file) {
            if let Some(existing) = selected_map.get_mut(&root.join(&targeted_test.test_file)) {
                // An independent group selected this test, so its execution is
                // not restricted to the configured-trigger target subset.
                let mut reasons_only = targeted_test.clone();
                reasons_only.targets.clear();
                merge_selected(existing, &reasons_only);
            }
            continue;
        }
        if let Some(existing) = candidates
            .iter_mut()
            .find(|candidate| candidate.test_file == targeted_test.test_file)
        {
            // A graph/direct reason independently selected the test; retain its
            // empty target list so finalization attaches every owning target.
            let mut reasons_only = targeted_test.clone();
            reasons_only.targets.clear();
            merge_selected(existing, &reasons_only);
        } else {
            candidates.push(targeted_test.clone());
        }
    }
}
