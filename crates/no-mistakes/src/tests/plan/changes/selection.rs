use super::*;

pub(in crate::tests::plan) fn apply(
    prepared: &PreparedTestPlanRequest,
    input: &ChangesInput,
    framework: Option<TestFramework>,
    discovered: DiscoveredTests,
    plan: &mut TestPlan,
) {
    let mut selection = Selection {
        prepared,
        input,
        plan,
        discovered: &discovered,
        selected: BTreeMap::new(),
        full_suite_fallback: false,
    };
    match &input.report {
        Ok(report) => selection.apply_categories(report, framework),
        Err(reason) => selection.fallback(reason.clone(), None),
    }
    selection.finish();
}

struct Selection<'a> {
    prepared: &'a PreparedTestPlanRequest,
    input: &'a ChangesInput,
    plan: &'a mut TestPlan,
    discovered: &'a DiscoveredTests,
    selected: BTreeMap<String, SelectedTest>,
    full_suite_fallback: bool,
}

impl Selection<'_> {
    fn apply_categories(&mut self, report: &ChangesReport, framework: Option<TestFramework>) {
        let frameworks = framework
            .as_ref()
            .map_or(TestFramework::value_variants(), std::slice::from_ref);
        for (name, changes) in &report.categories {
            let ids = changes
                .changed
                .iter()
                .chain(&changes.added)
                .chain(&changes.removed)
                .map(String::as_str)
                .collect::<BTreeSet<_>>();
            if ids.is_empty() {
                continue;
            }
            let category = &self.prepared.config.changes.categories[name];
            match CategoryMatches::find(self.prepared, self.discovered, name, category, &ids) {
                Ok(matched) => {
                    for (path, ids) in matched.tests {
                        self.select(
                            path,
                            ids.into_iter().map(|id| format!("changes:{name}:{id}")),
                            None,
                        );
                    }
                    if matched.unmatched.is_empty() {
                        continue;
                    }
                    let reason = format!(
                        "Unmatched IDs in changes category `{name}`: {}",
                        matched.unmatched.join(", ")
                    );
                    match matching::fallback_targets(
                        self.prepared,
                        frameworks,
                        category,
                        self.discovered,
                    ) {
                        Ok(targets) => self.fallback(reason, targets.as_ref()),
                        Err(error) => self.fallback(format!("{reason}. {error}"), None),
                    }
                }
                Err(error) => self.fallback(error, None),
            }
        }
        if !report.changed_files.is_empty() && !self.full_suite_fallback {
            if let Some(reason) =
                unplanned_changed_file_reason(self.prepared, framework, self.discovered, self.plan)
            {
                self.fallback(reason, None);
            }
        }
    }

    fn select(
        &mut self,
        path: &Path,
        via: impl IntoIterator<Item = String>,
        targets: Option<&matching::RunnerTargets>,
    ) {
        let test_file = relative_path(&self.prepared.root, path);
        let target_matches = |runner: &str, project: Option<&str>| {
            targets.is_none_or(|targets| {
                targets
                    .get(runner)
                    .is_some_and(|names| project.is_some_and(|name| names.contains(name)))
            })
        };
        let targets_for_test = self
            .discovered
            .targets_by_path
            .get(path)
            .into_iter()
            .flatten()
            .filter(|target| target_matches(&target.runner, target.project.as_deref()))
            .cloned()
            .collect::<Vec<_>>();
        if targets.is_some() && targets_for_test.is_empty() {
            return;
        }
        let reasons = via
            .into_iter()
            .map(|cause| ImpactReason {
                changed_file: self.input.file.clone(),
                path: vec![self.input.file.clone(), test_file.clone()],
                via: vec![cause],
                via_details: Vec::new(),
            })
            .collect();
        let next = SelectedTest {
            test_file: test_file.clone(),
            confidence: Confidence::High,
            reasons,
            targets: targets_for_test,
        };
        self.selected
            .entry(test_file)
            .and_modify(|test| merge_selected(test, &next))
            .or_insert(next);
    }

    fn fallback(&mut self, reason: String, targets: Option<&matching::RunnerTargets>) {
        self.full_suite_fallback |= targets.is_none();
        let scope = targets.map_or("the full discovered suite", |_| "configured runner targets");
        let warning = format!("{reason}. Selected {scope}.");
        self.plan.warnings.push(Warning {
            r#type: "changes-fallback".into(),
            message: warning,
            file: self.input.file.clone(),
            line: None,
        });
        self.plan.fallback_triggered = true;
        match &mut self.plan.fallback_reason {
            Some(existing) => {
                existing.push_str("; ");
                existing.push_str(&reason);
            }
            None => self.plan.fallback_reason = Some(reason),
        }
        for path in &self.discovered.tests {
            self.select(path, ["changes-fallback".into()], targets);
        }
    }

    fn finish(self) {
        if self.selected.is_empty() {
            return;
        }
        let selected = self.selected.keys().cloned().collect();
        let mut tests: BTreeMap<_, _> = std::mem::take(&mut self.plan.selected_tests)
            .into_iter()
            .map(|test| (test.test_file.clone(), test))
            .collect();
        for (path, mut test) in self.selected {
            test.targets.sort();
            test.targets.dedup();
            tests
                .entry(path)
                .and_modify(|old| merge_selected(old, &test))
                .or_insert(test);
        }
        self.plan.selected_tests = tests.into_values().collect();
        self.plan.groups.push(TestPlanGroupResult {
            r#type: "changes".into(),
            selected,
            remaining: self
                .discovered
                .tests
                .iter()
                .filter(|path| {
                    self.plan
                        .selected_tests
                        .binary_search_by(|test| {
                            test.test_file
                                .cmp(&relative_path(&self.prepared.root, path))
                        })
                        .is_err()
                })
                .count(),
            limit: None,
        });
    }
}
