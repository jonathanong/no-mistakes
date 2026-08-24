use super::plan::relative_path;
use super::prepared_plan::revisions::RevisionSources;
use super::Warning;
use no_mistakes::codebase::swift::{
    dependency_only_manifest_change, formatting_only_manifest_change, SwiftManifestDiagnostic,
};
use no_mistakes::config::v2::NoMistakesConfig;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Clone, Default)]
pub(crate) struct SwiftManifestAnalysis {
    pub(crate) seeds: Vec<PathBuf>,
    pub(crate) warnings: Vec<Warning>,
    pub(crate) fallback_triggered: bool,
    handled_files: HashSet<PathBuf>,
    dependency_only_files: HashSet<PathBuf>,
}
impl SwiftManifestAnalysis {
    pub(crate) fn handles(&self, path: &Path) -> bool {
        self.handled_files.contains(path)
    }
    pub(crate) fn dependency_only_files(&self) -> &HashSet<PathBuf> {
        &self.dependency_only_files
    }
}

pub(crate) fn analyze_swift_manifest_changes(
    root: &Path,
    config: &NoMistakesConfig,
    changed: &[PathBuf],
    revisions: &RevisionSources,
) -> SwiftManifestAnalysis {
    let mut analysis = SwiftManifestAnalysis::default();
    for package in &config.tests.swift.packages {
        let manifest = root
            .join(package.trim_end_matches('/'))
            .join("Package.swift");
        if !changed.contains(&manifest) {
            continue;
        }
        let file = relative_path(root, &manifest);
        let Some(base) = revisions.base_name() else {
            analysis.handled_files.insert(manifest.clone());
            warning(&mut analysis, "swift-manifest-no-baseline", format!("Could not determine old content of `{file}`. Provide `--base` to enable targeted Swift dependency analysis; full-suite selection requires global fallback opt-in."), file);
            continue;
        };
        let Some(before) = revisions.read_base(&manifest) else {
            analysis.handled_files.insert(manifest.clone());
            warning(&mut analysis, "swift-manifest-no-baseline", format!("Could not read `{file}` at base ref `{base}`; full-suite selection requires global fallback opt-in"), file);
            continue;
        };
        if revisions.is_diff_only() {
            analysis.handled_files.insert(manifest);
            warning(
                &mut analysis,
                "swift-manifest-no-baseline",
                format!(
                    "Could not determine new content of `{file}` in diff-only mode. Provide `--head` to enable targeted Swift dependency analysis; full-suite selection requires global fallback opt-in."
                ),
                file,
            );
            continue;
        }
        let after = revisions.read_after_or_empty(&manifest);
        if formatting_only_manifest_change(&before, &after) {
            analysis.handled_files.insert(manifest.clone());
            analysis.dependency_only_files.insert(manifest);
            continue;
        }
        match dependency_only_manifest_change(&before, &after) {
            Ok(true) => {
                analysis.handled_files.insert(manifest.clone());
                analysis.dependency_only_files.insert(manifest.clone());
                analysis.seeds.push(manifest);
            }
            Ok(false) => {}
            Err(SwiftManifestDiagnostic::UnsupportedDynamicDeclaration) => {
                analysis.handled_files.insert(manifest);
                warning(&mut analysis, "swift-manifest-unsupported-dynamic", format!("`{file}` has an unsupported dynamic dependency declaration; full-suite selection requires global fallback opt-in"), file)
            }
        }
    }
    analysis
}
fn warning(analysis: &mut SwiftManifestAnalysis, r#type: &str, message: String, file: String) {
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

    #[test]
    fn external_package_version_seeds_core_and_downstream_local_packages_before_sample() {
        let plan = changed_plan(
            "core-manifest-external.swift",
            "swift-clients/core/Package.swift",
            false,
        );
        assert!(!plan.fallback_triggered, "{plan:#?}");
        assert_eq!(
            dependency_group(&plan),
            [
                "swift-clients/android/Tests/AndroidTests/AndroidTests.swift",
                "swift-clients/core/Tests/CoreTests/CoreTests.swift",
                "swift-clients/ui/Tests/UITests/UITests.swift",
            ]
        );
        for selected in dependency_group(&plan) {
            let test = plan
                .selected_tests
                .iter()
                .find(|test| &test.test_file == selected)
                .expect("dependency selection must retain its causal test");
            assert!(test.reasons.iter().any(|reason| {
                reason.changed_file == "swift-clients/core/Package.swift"
                    && reason
                        .via
                        .iter()
                        .any(|via| via == "swift package dependency")
            }));
        }
    }

    #[test]
    fn local_package_product_binding_seeds_only_android() {
        let plan = changed_plan(
            "android-manifest-local.swift",
            "swift-clients/android/Package.swift",
            false,
        );
        assert!(!plan.fallback_triggered, "{plan:#?}");
        assert_eq!(
            dependency_group(&plan),
            ["swift-clients/android/Tests/AndroidTests/AndroidTests.swift"]
        );
    }

    #[test]
    fn mixed_manifest_stays_broad() {
        let plan = changed_plan(
            "core-manifest-mixed.swift",
            "swift-clients/core/Package.swift",
            false,
        );
        assert!(!plan.fallback_triggered, "{plan:#?}");
        assert!(plan
            .selected_tests
            .iter()
            .any(|test| test.reasons.iter().any(|reason| {
                reason.changed_file == "swift-clients/core/Package.swift"
                    && reason
                        .via
                        .first()
                        .is_some_and(|via| via == "swift package dependency")
            })));
    }

    #[test]
    fn formatting_only_manifest_has_no_causal_dependency_selection() {
        let plan = changed_plan(
            "core-manifest-formatting.swift",
            "swift-clients/core/Package.swift",
            false,
        );
        assert!(!plan.fallback_triggered, "{plan:#?}");
        assert!(dependency_group(&plan).is_empty(), "{plan:#?}");
    }

    #[test]
    fn unsupported_and_missing_baseline_honor_global_fallback_without_causal_selection() {
        for fallback in [false, true] {
            let unsupported = changed_plan(
                "core-manifest-dynamic.swift",
                "swift-clients/core/Package.swift",
                fallback,
            );
            assert_eq!(unsupported.fallback_triggered, fallback, "{unsupported:#?}");
            if !fallback {
                assert!(
                    dependency_group(&unsupported).is_empty(),
                    "{unsupported:#?}"
                );
                assert!(!sample_group(&unsupported).is_empty(), "{unsupported:#?}");
            }
            assert!(unsupported
                .warnings
                .iter()
                .any(|warning| warning.r#type == "swift-manifest-unsupported-dynamic"));

            let fixture = fixture();
            let root = fixture.path().canonicalize().unwrap();
            git_init(&root);
            git_commit_all(&root, "base");
            let missing = crate::tests::plan::generate_plan(&plan_args(
                &root,
                root.join("swift-clients/core/Package.swift"),
                None,
                fallback,
            ))
            .unwrap();
            assert_eq!(missing.fallback_triggered, fallback, "{missing:#?}");
            if !fallback {
                assert!(dependency_group(&missing).is_empty(), "{missing:#?}");
                assert!(!sample_group(&missing).is_empty(), "{missing:#?}");
            }
            assert!(missing
                .warnings
                .iter()
                .any(|warning| warning.r#type == "swift-manifest-no-baseline"));
        }
    }
}
