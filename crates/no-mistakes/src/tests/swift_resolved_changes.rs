use super::plan::relative_path;
use super::prepared_plan::revisions::RevisionSources;
use super::Warning;
use no_mistakes::codebase::swift::{
    diff_resolved_pins, parse_resolved_pins, SwiftResolvedDiagnostic,
};
use no_mistakes::config::v2::NoMistakesConfig;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Clone, Default)]
pub(crate) struct SwiftResolvedAnalysis {
    pub(crate) seeds: Vec<(PathBuf, PathBuf)>,
    pub(crate) warnings: Vec<Warning>,
    pub(crate) fallback_triggered: bool,
    handled_files: HashSet<PathBuf>,
}

impl SwiftResolvedAnalysis {
    pub(crate) fn handles(&self, path: &Path) -> bool {
        self.handled_files.contains(path)
    }
}

pub(crate) fn analyze_swift_resolved_changes(
    root: &Path,
    config: &NoMistakesConfig,
    changed_files: &[PathBuf],
    revisions: &RevisionSources,
) -> SwiftResolvedAnalysis {
    let mut analysis = SwiftResolvedAnalysis::default();
    for package in &config.tests.swift.packages {
        let package_root = root.join(package.trim_end_matches('/'));
        let resolved = package_root.join("Package.resolved");
        if !changed_files.contains(&resolved) {
            continue;
        }
        analysis.handled_files.insert(resolved.clone());
        let relative = relative_path(root, &resolved);
        let Some(base) = revisions.base_name() else {
            push_warning(
                &mut analysis,
                "swift-resolved-no-baseline",
                format!(
                    "Could not determine old content of `{relative}`. Provide `--base` to enable targeted Swift dependency analysis; full-suite selection requires global fallback opt-in."
                ),
                relative,
            );
            continue;
        };
        let Some(before) = revisions.read_base(&resolved) else {
            push_warning(
                &mut analysis,
                "swift-resolved-no-baseline",
                format!(
                    "Could not read `{relative}` at base ref `{base}`; full-suite selection requires global fallback opt-in"
                ),
                relative,
            );
            continue;
        };
        if revisions.is_diff_only() {
            push_warning(
                &mut analysis,
                "swift-resolved-no-baseline",
                format!(
                    "Could not determine new content of `{relative}` in diff-only mode. Provide `--head` to enable targeted Swift dependency analysis; full-suite selection requires global fallback opt-in."
                ),
                relative,
            );
            continue;
        }
        let after = revisions.read_after_or_empty(&resolved);
        let before = match parse_resolved_pins(&before) {
            Ok(pins) => pins,
            Err(diagnostic) => {
                push_diagnostic(&mut analysis, diagnostic, &relative, "base");
                continue;
            }
        };
        let after = match parse_resolved_pins(&after) {
            Ok(pins) => pins,
            Err(diagnostic) => {
                push_diagnostic(&mut analysis, diagnostic, &relative, "head");
                continue;
            }
        };
        if !diff_resolved_pins(&before, &after).is_empty() {
            analysis
                .seeds
                .push((resolved, package_root.join("Package.swift")));
        }
    }
    analysis
}

fn push_diagnostic(
    analysis: &mut SwiftResolvedAnalysis,
    diagnostic: SwiftResolvedDiagnostic,
    file: &str,
    revision: &str,
) {
    let (r#type, description) = match diagnostic {
        SwiftResolvedDiagnostic::Malformed => ("swift-resolved-malformed", "malformed"),
        SwiftResolvedDiagnostic::UnsupportedSchema => (
            "swift-resolved-unsupported-schema",
            "uses an unsupported schema",
        ),
    };
    push_warning(
        analysis,
        r#type,
        format!(
            "`{file}` at {revision} is {description}; full-suite selection requires global fallback opt-in"
        ),
        file.to_string(),
    );
}

fn push_warning(analysis: &mut SwiftResolvedAnalysis, r#type: &str, message: String, file: String) {
    analysis.warnings.push(Warning {
        r#type: r#type.to_string(),
        message,
        file,
        line: None,
    });
    analysis.fallback_triggered = true;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{git_commit_all, git_init, materialize_saved_fixture};
    use crate::tests::{PlanArgs, TestFramework};

    fn plan_args(root: &Path, changed: PathBuf, base: Option<&str>, fallback: bool) -> PlanArgs {
        PlanArgs {
            framework: Some(TestFramework::Swift),
            root: root.to_path_buf(),
            config: None,
            tsconfig: None,
            base: base.map(str::to_string),
            head: None,
            from_git_diff: None,
            changed_file: vec![changed],
            changed_files: None,
            diff: None,
            diff_stdin: false,
            diff_command: None,
            entrypoints: Vec::new(),
            entrypoint_symbols: Vec::new(),
            include_symbols: false,
            diff_content: None,
            environment: "pre-push".to_string(),
            limit_percent: None,
            limit_files: None,
            global_config_fallback: Some(fallback),
            direct_test_owner: false,
            format: None,
            json: false,
            include_comment: false,
            include_glob: Vec::new(),
        }
    }

    fn fixture() -> tempfile::TempDir {
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/swift-resolved-plan/fixture");
        materialize_saved_fixture(&source)
    }

    fn changed_plan(change: &str, target: &str, fallback: bool) -> crate::tests::TestPlan {
        let fixture = fixture();
        let root = fixture.path().canonicalize().unwrap();
        git_init(&root);
        git_commit_all(&root, "base");
        std::fs::copy(root.join("changes").join(change), root.join(target)).unwrap();
        crate::tests::plan::generate_plan(&plan_args(
            &root,
            root.join(target),
            Some("HEAD"),
            fallback,
        ))
        .unwrap()
    }

    fn dependency_group(plan: &crate::tests::TestPlan) -> &[String] {
        &plan
            .groups
            .iter()
            .find(|group| group.r#type == "dependencies")
            .expect("dependencies group must be reported")
            .selected
    }

    fn sample_group(plan: &crate::tests::TestPlan) -> &[String] {
        &plan
            .groups
            .iter()
            .find(|group| group.r#type == "sample")
            .expect("sample group must be reported")
            .selected
    }

    fn assert_dependency_causality(plan: &crate::tests::TestPlan, changed_file: &str) {
        for selected in dependency_group(plan) {
            let test = plan
                .selected_tests
                .iter()
                .find(|test| &test.test_file == selected)
                .expect("dependency selection must retain its causal test");
            assert!(test.reasons.iter().any(|reason| {
                reason.changed_file == changed_file
                    && reason
                        .via
                        .first()
                        .is_some_and(|via| via == "swift package dependency")
            }));
        }
    }

    #[test]
    fn core_resolved_pin_selects_core_and_downstream_local_packages_before_sample() {
        let plan = changed_plan(
            "core-changed.json",
            "swift-clients/core/Package.resolved",
            false,
        );
        assert!(!plan.fallback_triggered, "{plan:#?}");
        assert!(plan.warnings.is_empty(), "{plan:#?}");
        assert_eq!(
            dependency_group(&plan),
            [
                "swift-clients/android/Tests/AndroidTests/AndroidTests.swift",
                "swift-clients/core/Tests/CoreTests/CoreTests.swift",
                "swift-clients/ui/Tests/UITests/UITests.swift",
            ]
        );
        assert_dependency_causality(&plan, "swift-clients/core/Package.resolved");
    }

    #[test]
    fn android_resolved_pin_stays_with_its_own_package() {
        let plan = changed_plan(
            "android-changed.json",
            "swift-clients/android/Package.resolved",
            false,
        );
        assert!(!plan.fallback_triggered, "{plan:#?}");
        assert_eq!(
            dependency_group(&plan),
            ["swift-clients/android/Tests/AndroidTests/AndroidTests.swift"],
            "{plan:#?}"
        );
    }

    #[test]
    fn ui_resolved_pin_selects_ui_and_downstream_android_but_not_upstream_core() {
        let plan = changed_plan(
            "ui-changed.json",
            "swift-clients/ui/Package.resolved",
            false,
        );
        assert!(!plan.fallback_triggered, "{plan:#?}");
        assert!(plan.warnings.is_empty(), "{plan:#?}");
        assert_eq!(
            dependency_group(&plan),
            [
                "swift-clients/android/Tests/AndroidTests/AndroidTests.swift",
                "swift-clients/ui/Tests/UITests/UITests.swift",
            ],
            "{plan:#?}"
        );
        assert_dependency_causality(&plan, "swift-clients/ui/Package.resolved");
    }

    #[test]
    fn untraceable_resolved_pin_warns_and_keeps_the_safety_sample_without_global_fallback() {
        let plan = changed_plan(
            "orphan-changed.json",
            "swift-clients/orphan/Package.resolved",
            false,
        );
        assert!(!plan.fallback_triggered, "{plan:#?}");
        assert!(dependency_group(&plan).is_empty(), "{plan:#?}");
        assert!(plan
            .warnings
            .iter()
            .any(|warning| warning.r#type == "swift-dependency-untraceable"));
        assert!(!sample_group(&plan).is_empty(), "{plan:#?}");
        assert!(plan.selected_tests.iter().all(|test| test
            .reasons
            .iter()
            .all(|reason| reason.changed_file != "swift-clients/orphan/Package.resolved")));
    }

    #[test]
    fn untraceable_resolved_pin_uses_full_fallback_only_when_enabled() {
        let plan = changed_plan(
            "orphan-changed.json",
            "swift-clients/orphan/Package.resolved",
            true,
        );
        assert!(plan.fallback_triggered, "{plan:#?}");
        assert_eq!(plan.selected_tests.len(), 3, "{plan:#?}");
        assert!(plan
            .warnings
            .iter()
            .any(|warning| warning.r#type == "swift-dependency-untraceable"));
    }

    #[test]
    fn plain_plan_untraceable_resolved_pin_obeys_global_fallback_policy() {
        for (fallback, expected_fallback, expected_tests) in [(false, false, 0), (true, true, 3)] {
            let fixture = fixture();
            let root = fixture.path().canonicalize().unwrap();
            git_init(&root);
            git_commit_all(&root, "base");
            let changed = root.join("swift-clients/orphan/Package.resolved");
            std::fs::copy(root.join("changes/orphan-changed.json"), &changed).unwrap();
            let mut args = plan_args(&root, changed, Some("HEAD"), fallback);
            args.framework = None;
            let plan = crate::tests::plan::generate_plan(&args).unwrap();
            assert_eq!(plan.fallback_triggered, expected_fallback, "{plan:#?}");
            assert_eq!(plan.selected_tests.len(), expected_tests, "{plan:#?}");
            assert!(plan
                .warnings
                .iter()
                .any(|warning| warning.r#type == "swift-dependency-untraceable"));
        }
    }

    #[test]
    fn semantic_seed_synthesizes_dependencies_before_a_custom_sample_only_group() {
        let fixture = fixture();
        let root = fixture.path().canonicalize().unwrap();
        let config_path = root.join(".no-mistakes.yml");
        std::fs::copy(
            root.join("changes/sample-only.no-mistakes.yml"),
            &config_path,
        )
        .unwrap();
        git_init(&root);
        git_commit_all(&root, "base");
        std::fs::copy(
            root.join("changes/core-changed.json"),
            root.join("swift-clients/core/Package.resolved"),
        )
        .unwrap();

        let plan = crate::tests::plan::generate_plan(&plan_args(
            &root,
            root.join("swift-clients/core/Package.resolved"),
            Some("HEAD"),
            false,
        ))
        .unwrap();

        assert_eq!(
            dependency_group(&plan),
            [
                "swift-clients/android/Tests/AndroidTests/AndroidTests.swift",
                "swift-clients/core/Tests/CoreTests/CoreTests.swift",
                "swift-clients/ui/Tests/UITests/UITests.swift",
            ],
            "{plan:#?}"
        );
        assert_eq!(plan.groups[0].r#type, "dependencies", "{plan:#?}");
    }

    #[test]
    fn plain_plan_keeps_semantic_native_seed_causality() {
        let fixture = fixture();
        let root = fixture.path().canonicalize().unwrap();
        git_init(&root);
        git_commit_all(&root, "base");
        let changed = root.join("swift-clients/core/Package.resolved");
        std::fs::copy(root.join("changes/core-changed.json"), &changed).unwrap();
        let mut args = plan_args(&root, changed, Some("HEAD"), false);
        args.framework = None;

        let plan = crate::tests::plan::generate_plan(&args).unwrap();

        assert!(!plan.fallback_triggered, "{plan:#?}");
        assert_eq!(
            plan.selected_tests
                .iter()
                .map(|test| test.test_file.as_str())
                .collect::<Vec<_>>(),
            [
                "swift-clients/android/Tests/AndroidTests/AndroidTests.swift",
                "swift-clients/core/Tests/CoreTests/CoreTests.swift",
                "swift-clients/ui/Tests/UITests/UITests.swift",
            ],
            "{plan:#?}"
        );
        assert!(plan.selected_tests.iter().all(|test| {
            test.reasons.iter().any(|reason| {
                reason.changed_file == "swift-clients/core/Package.resolved"
                    && reason
                        .via
                        .first()
                        .is_some_and(|via| via == "swift package dependency")
            })
        }));
    }

    #[test]
    fn invalid_resolved_uses_environment_fallback_policy_without_causal_selection() {
        for (change, warning_type) in [
            ("malformed.json", "swift-resolved-malformed"),
            (
                "unsupported-schema.json",
                "swift-resolved-unsupported-schema",
            ),
        ] {
            let scoped = changed_plan(change, "swift-clients/core/Package.resolved", false);
            assert!(!scoped.fallback_triggered, "{scoped:#?}");
            assert!(dependency_group(&scoped).is_empty(), "{scoped:#?}");
            assert!(scoped
                .warnings
                .iter()
                .any(|warning| warning.r#type == warning_type));

            let global = changed_plan(change, "swift-clients/core/Package.resolved", true);
            assert!(global.fallback_triggered, "{global:#?}");
            assert!(global
                .warnings
                .iter()
                .any(|warning| warning.r#type == warning_type));
        }
    }

    #[test]
    fn missing_baseline_uses_environment_fallback_policy_without_causal_selection() {
        let fixture = fixture();
        let root = fixture.path().canonicalize().unwrap();
        let changed = root.join("swift-clients/core/Package.resolved");
        let scoped =
            crate::tests::plan::generate_plan(&plan_args(&root, changed.clone(), None, false))
                .unwrap();
        assert!(!scoped.fallback_triggered, "{scoped:#?}");
        assert!(dependency_group(&scoped).is_empty(), "{scoped:#?}");
        assert!(scoped
            .warnings
            .iter()
            .any(|warning| warning.r#type == "swift-resolved-no-baseline"));

        let global =
            crate::tests::plan::generate_plan(&plan_args(&root, changed, None, true)).unwrap();
        assert!(global.fallback_triggered, "{global:#?}");
        assert!(global
            .warnings
            .iter()
            .any(|warning| warning.r#type == "swift-resolved-no-baseline"));
    }
}
