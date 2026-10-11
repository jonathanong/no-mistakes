use super::*;

pub(super) fn global_fallback_plan(
    args: &PlanArgs,
    prepared: &crate::tests::prepared_plan::PreparedTestPlanRequest,
    changed_files: &[PathBuf],
) -> Option<TestPlan> {
    let root = &prepared.root;
    let lockfile_analysis = &prepared.lockfile_analysis;
    // 2b. Determine fallback trigger.
    //
    // Every full-suite fallback is explicit opt-in, including diff-only and
    // binary lockfiles whose contents cannot be analyzed.
    let fallback_reason = if global_config_fallback(args)
        && (lockfile_analysis.fallback_triggered
            || prepared.package_manifest_analysis.fallback_triggered
            || prepared.swift_resolved_analysis.fallback_triggered
            || prepared.swift_manifest_analysis.fallback_triggered
            || prepared.dotnet_dependency_analysis.fallback_triggered)
    {
        lockfile_analysis
            .warnings
            .first()
            .or_else(|| prepared.package_manifest_analysis.warnings.first())
            .or_else(|| prepared.swift_resolved_analysis.warnings.first())
            .or_else(|| prepared.swift_manifest_analysis.warnings.first())
            .or_else(|| prepared.dotnet_dependency_analysis.warnings.first())
            .map(|warning| (warning.message.clone(), root.join(&warning.file)))
    } else if global_config_fallback(args) {
        global_config_trigger(root, changed_files, None, prepared)
    } else {
        None
    };

    if let Some((reason, trigger_file)) = fallback_reason {
        let relative_changed = relative_path(root, &trigger_file);
        let all_test_files = global_config::discover_all_tests_from_prepared(prepared);
        let mut selected_tests = Vec::new();
        for test in all_test_files {
            let rel_test = relative_path(root, &test);
            selected_tests.push(SelectedTest {
                test_file: rel_test.clone(),
                confidence: Confidence::High,
                targets: Vec::new(),
                reasons: vec![ImpactReason {
                    changed_file: relative_changed.clone(),
                    path: vec![relative_changed.clone(), rel_test],
                    via: vec!["global configuration".to_string()],
                    via_details: Vec::new(),
                }],
            });
        }
        selected_tests.sort_by(|a, b| a.test_file.cmp(&b.test_file));
        return Some(TestPlan {
            changed_files: Vec::new(),
            selected_tests,
            groups: Vec::new(),
            warnings: {
                let mut warnings = lockfile_analysis.warnings.clone();
                warnings.extend(prepared.package_manifest_analysis.warnings.iter().cloned());
                warnings.extend(prepared.swift_resolved_analysis.warnings.iter().cloned());
                warnings.extend(prepared.swift_manifest_analysis.warnings.iter().cloned());
                warnings.extend(prepared.dotnet_dependency_analysis.warnings.iter().cloned());
                warnings.extend(prepared.tsconfig_warnings());
                warnings
            },
            fallback_triggered: true,
            fallback_reason: Some(reason),
            ..Default::default()
        });
    }

    None
}
