use super::*;

pub(super) fn generate_framework_plan(
    args: &PlanArgs,
    prepared: &crate::tests::prepared_plan::PreparedTestPlanRequest,
    framework: crate::tests::TestFramework,
    timing: Option<&mut crate::impacted_checks::timing::TimingTracker>,
) -> Result<TestPlan> {
    let root = &prepared.root;
    let config = &prepared.config;
    let collected = &prepared.collected;
    let changed_files = prepared.planning_changed_files(Some(framework));
    let deleted_files = &collected.deleted;
    let lockfile_analysis = &prepared.lockfile_analysis;
    if args.direct_test_owner {
        return crate::tests::configured_plan::generate_direct_test_owner_plan_with_prepared(
            framework, prepared,
        );
    }
    // Compute lockfile changed packages for BFS tracing in framework plans — same
    // structure as the non-framework §4b path below. Parseable lockfile diffs no
    // longer force an unconditional full-suite fallback; we wire the packages into
    // the configured-plan dependencies group instead.
    // fallback_triggered means binary / invalid-ref / diff-only — no parseable diff available.
    // Full-suite selection still requires the effective global fallback opt-in.
    let javascript_dependencies =
        crate::tests::prepared_plan::javascript_dependency_framework(Some(framework));
    let forced_fallback = prepared
        .framework_config_trigger(framework)
        .or_else(|| {
            global_config::excluding_v2_config(root, &collected.files, Some(framework), prepared)
        })
        .or_else(|| {
            if javascript_dependencies && lockfile_analysis.fallback_triggered {
                lockfile_analysis
                    .warnings
                    .first()
                    .map(|w| (w.message.clone(), root.join(&w.file)))
            } else {
                None
            }
        })
        .or_else(|| {
            (javascript_dependencies && prepared.package_manifest_analysis.fallback_triggered).then(
                || {
                    let warning = prepared
                        .package_manifest_analysis
                        .warnings
                        .first()
                        .expect("package manifest fallback warning");
                    (warning.message.clone(), root.join(&warning.file))
                },
            )
        })
        .or_else(|| {
            if framework == crate::tests::TestFramework::Swift
                && prepared.swift_resolved_analysis.fallback_triggered
            {
                prepared
                    .swift_resolved_analysis
                    .warnings
                    .first()
                    .map(|warning| (warning.message.clone(), root.join(&warning.file)))
            } else {
                None
            }
        })
        .or_else(|| {
            (framework == crate::tests::TestFramework::Swift
                && prepared.swift_manifest_analysis.fallback_triggered)
                .then(|| {
                    let warning = prepared
                        .swift_manifest_analysis
                        .warnings
                        .first()
                        .expect("fallback warning");
                    (warning.message.clone(), root.join(&warning.file))
                })
        })
        .or_else(|| {
            (framework == crate::tests::TestFramework::Dotnet
                && prepared.dotnet_dependency_analysis.fallback_triggered)
                .then(|| {
                    let warning = prepared
                        .dotnet_dependency_analysis
                        .warnings
                        .first()
                        .expect(".NET dependency fallback warning");
                    (warning.message.clone(), root.join(&warning.file))
                })
        });
    let discovered_tests = crate::tests::configured_plan::discover_framework_tests_from_prepared(
        args, framework, prepared,
    )?;
    let mut plan = crate::tests::configured_plan::generate_configured_plan_with_prepared(
        args,
        framework,
        root,
        config,
        &changed_files,
        deleted_files,
        &collected.diff_files,
        if javascript_dependencies {
            &prepared.lockfile_changed_packages
        } else {
            &[]
        },
        &prepared.workspace_map,
        forced_fallback,
        discovered_tests,
        prepared,
        timing,
    )?;
    let mut warning_keys: HashSet<WarningKey> = plan.warnings.iter().map(warning_key).collect();
    let dependency_warnings = javascript_dependencies
        .then_some(lockfile_analysis.warnings.iter())
        .into_iter()
        .flatten()
        .chain(
            javascript_dependencies
                .then_some(prepared.package_manifest_analysis.warnings.iter())
                .into_iter()
                .flatten(),
        )
        .chain(
            (framework == crate::tests::TestFramework::Swift)
                .then_some(prepared.swift_resolved_analysis.warnings.iter())
                .into_iter()
                .flatten(),
        )
        .chain(
            (framework == crate::tests::TestFramework::Swift)
                .then_some(prepared.swift_manifest_analysis.warnings.iter())
                .into_iter()
                .flatten(),
        )
        .chain(
            (framework == crate::tests::TestFramework::Dotnet)
                .then_some(prepared.dotnet_dependency_analysis.warnings.iter())
                .into_iter()
                .flatten(),
        );
    for warning in dependency_warnings {
        if warning_keys.insert(warning_key(warning)) {
            plan.warnings.push(warning.clone());
        }
    }
    Ok(plan)
}
