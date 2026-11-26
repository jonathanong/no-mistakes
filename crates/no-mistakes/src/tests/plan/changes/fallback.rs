use super::*;

pub(super) fn unplanned_changed_file_reason(
    prepared: &PreparedTestPlanRequest,
    framework: Option<TestFramework>,
    discovered: &DiscoveredTests,
    plan: &TestPlan,
) -> Option<String> {
    let config_fallback = match framework {
        Some(fw) => prepared.framework_config_trigger(fw).or_else(|| {
            super::super::global_config::excluding_v2_config(
                &prepared.root,
                &prepared.collected.files,
                Some(fw),
                prepared,
            )
        }),
        None => super::super::global_config::global_config_trigger(
            &prepared.root,
            &prepared.collected.files,
            None,
            prepared,
        ),
    };
    if let Some((reason, _)) = config_fallback {
        return Some(reason);
    }
    if let Some(deleted) = prepared.collected.deleted.first() {
        return Some(format!(
            "Deleted file in changes report: {}",
            relative_path(&prepared.root, deleted)
        ));
    }
    if let Some(reason) = dependency_fallback_reason(prepared, framework) {
        return Some(reason);
    }
    if let Some(reason) = plan.warnings.iter().find_map(|warning| {
        matches!(
            warning.r#type.as_str(),
            "fallback-warning"
                | "deleted-file"
                | "package-dependency-untraceable"
                | "swift-dependency-untraceable"
                | "dotnet-dependency-untraceable"
        )
        .then(|| warning.message.clone())
    }) {
        return Some(reason);
    }
    let skipped_dep_seeds = plan.fallback_triggered
        && plan.selected_tests.len() < discovered.tests.len()
        && !plan
            .groups
            .iter()
            .any(|group| group.r#type == "dependencies");
    if skipped_dep_seeds && crate::tests::prepared_plan::javascript_dependency_framework(framework)
    {
        return crate::tests::configured_plan::lockfile_seeds::first_untraceable_lockfile_dependency(
            prepared,
            &discovered.tests,
        );
    }
    None
}

fn dependency_fallback_reason(
    prepared: &PreparedTestPlanRequest,
    framework: Option<TestFramework>,
) -> Option<String> {
    let js = crate::tests::prepared_plan::javascript_dependency_framework(framework);
    let swift = matches!(framework, None | Some(TestFramework::Swift));
    let dotnet = matches!(framework, None | Some(TestFramework::Dotnet));
    [
        (
            js && prepared.lockfile_analysis.fallback_triggered,
            &prepared.lockfile_analysis.warnings,
        ),
        (
            js && prepared.package_manifest_analysis.fallback_triggered,
            &prepared.package_manifest_analysis.warnings,
        ),
        (
            swift && prepared.swift_resolved_analysis.fallback_triggered,
            &prepared.swift_resolved_analysis.warnings,
        ),
        (
            swift && prepared.swift_manifest_analysis.fallback_triggered,
            &prepared.swift_manifest_analysis.warnings,
        ),
        (
            dotnet && prepared.dotnet_dependency_analysis.fallback_triggered,
            &prepared.dotnet_dependency_analysis.warnings,
        ),
    ]
    .into_iter()
    .find_map(|(triggered, warnings)| {
        triggered
            .then(|| warnings.first().map(|warning| warning.message.clone()))
            .flatten()
    })
}
