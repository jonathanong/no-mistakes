use super::plan::relative_path;
use super::prepared_plan::revisions::RevisionSources;
use super::Warning;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

mod classify;
use classify::{classify_change, PackageManifestChange};

#[derive(Clone, Default)]
pub(crate) struct PackageManifestAnalysis {
    /// Dependency names paired with the changed manifest that declared them.
    /// A package name alone is not a sound seed in a monorepo: two workspaces
    /// can both depend on it while only one declaration changed.
    pub(crate) changed_packages: Vec<(String, String, PathBuf)>,
    pub(crate) warnings: Vec<Warning>,
    pub(crate) fallback_triggered: bool,
    broad_trigger_exclusions: HashSet<PathBuf>,
    handled_files: HashSet<PathBuf>,
}

impl PackageManifestAnalysis {
    pub(crate) fn excludes_broad_trigger(&self, path: &Path) -> bool {
        self.broad_trigger_exclusions.contains(path)
    }

    pub(crate) fn handles(&self, path: &Path) -> bool {
        self.handled_files.contains(path)
    }
}

pub(crate) fn analyze_package_manifest_changes(
    root: &Path,
    changed_files: &[PathBuf],
    revisions: &RevisionSources,
    workspace_map: &no_mistakes::codebase::workspaces::WorkspaceMap,
) -> PackageManifestAnalysis {
    let mut analysis = PackageManifestAnalysis::default();
    for manifest in changed_files
        .iter()
        .filter(|path| path.file_name().and_then(|name| name.to_str()) == Some("package.json"))
        .filter(|path| {
            **path == root.join("package.json")
                || workspace_map
                    .packages
                    .iter()
                    .any(|package| **path == package.dir.join("package.json"))
        })
    {
        let file = relative_path(root, manifest);
        let Some(base) = revisions.base_name() else {
            diagnose(
                &mut analysis,
                manifest,
                &file,
                "package-manifest-no-baseline",
                format!("Could not determine old content of `{file}`. Provide `--base` to enable targeted package dependency analysis; full-suite selection requires global fallback opt-in."),
            );
            continue;
        };
        let Some(before) = revisions.read_base(manifest) else {
            diagnose(
                &mut analysis,
                manifest,
                &file,
                "package-manifest-no-baseline",
                format!("Could not read `{file}` at base ref `{base}`; full-suite selection requires global fallback opt-in"),
            );
            continue;
        };
        let after = revisions.read_after(manifest);
        let Some(after) = after else {
            let detail = if revisions.is_diff_only() {
                "Could not determine new content in diff-only mode. Provide `--head` to enable targeted package dependency analysis"
            } else {
                "Could not read the requested head"
            };
            diagnose(
                &mut analysis,
                manifest,
                &file,
                "package-manifest-no-baseline",
                format!(
                    "{detail} for `{file}`; full-suite selection requires global fallback opt-in"
                ),
            );
            continue;
        };
        match classify_change(&before, &after) {
            Ok(PackageManifestChange::DependencyOnly(packages)) => {
                analysis.broad_trigger_exclusions.insert(manifest.clone());
                analysis.handled_files.insert(manifest.clone());
                analysis.changed_packages.extend(
                    packages
                        .into_iter()
                        .map(|package| (package, file.clone(), manifest.clone())),
                );
            }
            Ok(PackageManifestChange::FormattingOnly) => {
                analysis.broad_trigger_exclusions.insert(manifest.clone());
                analysis.handled_files.insert(manifest.clone());
            }
            Ok(PackageManifestChange::Broad) => {}
            Err(()) => diagnose(
                &mut analysis,
                manifest,
                &file,
                "package-manifest-malformed",
                format!("`{file}` is malformed at a compared revision; full-suite selection requires global fallback opt-in"),
            ),
        }
    }
    analysis
}

fn diagnose(
    analysis: &mut PackageManifestAnalysis,
    manifest: &Path,
    file: &str,
    r#type: &str,
    message: String,
) {
    analysis
        .broad_trigger_exclusions
        .insert(manifest.to_path_buf());
    analysis.handled_files.insert(manifest.to_path_buf());
    analysis.warnings.push(Warning {
        r#type: r#type.to_string(),
        message,
        file: file.to_string(),
        line: None,
    });
    analysis.fallback_triggered = true;
}

#[cfg(test)]
#[path = "package_manifest_changes/workspace_scope_tests.rs"]
mod workspace_scope_tests;

#[cfg(test)]
#[path = "package_manifest_changes/pnpm_installation_sections_tests.rs"]
mod pnpm_installation_sections_tests;

#[cfg(test)]
#[path = "package_manifest_changes/pnpm_traceability_tests.rs"]
mod pnpm_traceability_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{git_commit_all, git_init, materialize_saved_fixture};
    use crate::tests::{PlanArgs, TestFramework};
    use std::collections::BTreeSet;

    fn fixture(name: &str) -> String {
        std::fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../fixtures/test-plan/package-manifest-diff")
                .join(name),
        )
        .unwrap()
    }

    #[test]
    fn classifies_supported_dependency_maps_without_hiding_mixed_configuration() {
        let base = fixture("base.json");
        assert_eq!(
            classify_change(&base, &fixture("dependencies.json")),
            Ok(PackageManifestChange::DependencyOnly(BTreeSet::from([
                "alpha".to_string(),
                "dev".to_string(),
                "optional".to_string(),
                "peer".to_string(),
            ])))
        );
        let changed_names = BTreeSet::from([
            "alpha".to_string(),
            "dev".to_string(),
            "optional".to_string(),
            "peer".to_string(),
        ]);
        assert_eq!(
            classify_change(&base, &fixture("empty.json")),
            Ok(PackageManifestChange::DependencyOnly(changed_names.clone()))
        );
        assert_eq!(
            classify_change(&fixture("empty.json"), &base),
            Ok(PackageManifestChange::DependencyOnly(changed_names))
        );
        assert_eq!(
            classify_change(&base, &fixture("formatting.json")),
            Ok(PackageManifestChange::FormattingOnly)
        );
        assert_eq!(
            classify_change(&base, &fixture("mixed.json")),
            Ok(PackageManifestChange::Broad)
        );
        assert_eq!(
            classify_change(&base, &fixture("package-manager.json")),
            Ok(PackageManifestChange::Broad)
        );
        assert_eq!(
            classify_change(&base, &fixture("peer-meta.json")),
            Ok(PackageManifestChange::Broad)
        );
        assert_eq!(classify_change(&base, &fixture("malformed.json")), Err(()));
        assert_eq!(
            classify_change(&base, r#"{"name":"fixture","dependencies":"invalid"}"#),
            Err(())
        );
        assert_eq!(
            classify_change(r#"{"name":"fixture","dependencies":"invalid"}"#, &base),
            Err(())
        );
    }

    fn plan(change: Option<&str>, base: Option<&str>, fallback: bool) -> crate::tests::TestPlan {
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/package-manifest-plan/fixture");
        let fixture = materialize_saved_fixture(&source);
        let root = fixture.path().canonicalize().unwrap();
        git_init(&root);
        git_commit_all(&root, "base");
        if let Some(change) = change {
            std::fs::copy(root.join("changes").join(change), root.join("package.json")).unwrap();
        } else {
            std::fs::remove_file(root.join("package.json")).unwrap();
        }
        crate::tests::plan::generate_plan(&plan_args(
            &root,
            vec![root.join("package.json")],
            base,
            fallback,
        ))
        .unwrap()
    }

    fn plan_args(
        root: &Path,
        changed_file: Vec<PathBuf>,
        base: Option<&str>,
        fallback: bool,
    ) -> PlanArgs {
        PlanArgs {
            framework: Some(TestFramework::Vitest),
            root: root.to_path_buf(),
            config: None,
            tsconfig: None,
            base: base.map(str::to_string),
            head: None,
            from_git_diff: None,
            changed_file,
            changed_files: None,
            diff: None,
            diff_stdin: false,
            diff_command: None,
            entrypoints: Vec::new(),
            entrypoint_symbols: Vec::new(),
            include_symbols: false,
            diff_content: None,
            environment: "prePush".to_string(),
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

    fn group<'a>(plan: &'a crate::tests::TestPlan, kind: &str) -> Option<&'a [String]> {
        plan.groups
            .iter()
            .find(|group| group.r#type == kind)
            .map(|group| group.selected.as_slice())
    }

    #[test]
    fn dependency_only_manifest_is_causal_before_the_safety_sample() {
        let plan = plan(Some("dependencies.json"), Some("HEAD"), true);
        assert!(!plan.fallback_triggered, "{plan:#?}");
        assert_eq!(
            group(&plan, "dependencies"),
            Some(&["alpha.test.ts".to_string()][..])
        );
        assert_eq!(group(&plan, "sample").map(<[String]>::len), Some(1));
        assert!(plan.warnings.is_empty(), "{plan:#?}");
    }

    #[test]
    fn manifest_and_lockfile_changes_merge_reasons_for_the_same_package() {
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/package-manifest-plan/fixture");
        let fixture = materialize_saved_fixture(&source);
        let root = fixture.path().canonicalize().unwrap();
        git_init(&root);
        git_commit_all(&root, "base");
        std::fs::copy(
            root.join("changes/dependencies.json"),
            root.join("package.json"),
        )
        .unwrap();
        std::fs::copy(
            root.join("changes/pnpm-lock.yaml"),
            root.join("pnpm-lock.yaml"),
        )
        .unwrap();
        let plan = crate::tests::plan::generate_plan(&plan_args(
            &root,
            vec![root.join("package.json"), root.join("pnpm-lock.yaml")],
            Some("HEAD"),
            false,
        ))
        .unwrap();
        assert_eq!(
            group(&plan, "dependencies"),
            Some(&["alpha.test.ts".to_string()][..])
        );
        let alpha = plan
            .selected_tests
            .iter()
            .find(|test| test.test_file == "alpha.test.ts")
            .unwrap();
        let changed_files: BTreeSet<_> = alpha
            .reasons
            .iter()
            .map(|reason| reason.changed_file.as_str())
            .collect();
        assert_eq!(
            changed_files,
            BTreeSet::from(["package.json", "pnpm-lock.yaml"])
        );
    }

    #[test]
    fn workspace_manifest_dependency_delta_does_not_cross_seed_same_package_consumers() {
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/package-manifest-plan/fixture");
        let fixture = materialize_saved_fixture(&source);
        let root = fixture.path().canonicalize().unwrap();
        git_init(&root);
        git_commit_all(&root, "base");
        let manifest = root.join("workspaces/a/package.json");
        std::fs::copy(root.join("changes/workspace-a-package.json"), &manifest).unwrap();
        let plan = crate::tests::plan::generate_plan(&plan_args(
            &root,
            vec![manifest],
            Some("HEAD"),
            false,
        ))
        .unwrap();
        assert_eq!(
            group(&plan, "dependencies"),
            Some(
                &[
                    "workspaces/a/alpha.test.ts".to_string(),
                    "workspaces/b/alpha.test.ts".to_string(),
                ][..]
            ),
            "{plan:#?}"
        );
    }

    #[test]
    fn malformed_and_baseline_less_manifests_warn_and_obey_fallback_policy() {
        for (change, base, warning_type) in [
            (
                Some("malformed.json"),
                Some("HEAD"),
                "package-manifest-malformed",
            ),
            (
                Some("dependencies.json"),
                None,
                "package-manifest-no-baseline",
            ),
            (None, Some("HEAD"), "package-manifest-no-baseline"),
        ] {
            let targeted = plan(change, base, false);
            assert!(!targeted.fallback_triggered, "{targeted:#?}");
            assert_eq!(
                group(&targeted, "dependencies"),
                Some(&[][..]),
                "{targeted:#?}"
            );
            assert_eq!(group(&targeted, "sample").map(<[String]>::len), Some(1));
            assert!(
                targeted
                    .warnings
                    .iter()
                    .any(|warning| warning.r#type == warning_type),
                "{targeted:#?}"
            );

            let fallback = plan(change, base, true);
            assert!(fallback.fallback_triggered, "{fallback:#?}");
            assert_eq!(fallback.selected_tests.len(), 5, "{fallback:#?}");
            assert!(
                fallback
                    .warnings
                    .iter()
                    .any(|warning| warning.r#type == warning_type),
                "{fallback:#?}"
            );
        }
    }

    #[test]
    fn formatting_is_ignored_while_mixed_configuration_remains_broad() {
        let formatting = plan(Some("formatting.json"), Some("HEAD"), true);
        assert!(!formatting.fallback_triggered, "{formatting:#?}");
        assert_eq!(group(&formatting, "dependencies"), Some(&[][..]));
        assert_eq!(group(&formatting, "sample").map(<[String]>::len), Some(1));

        let mixed = plan(Some("mixed.json"), Some("HEAD"), true);
        assert!(mixed.fallback_triggered, "{mixed:#?}");
        assert_eq!(mixed.selected_tests.len(), 5, "{mixed:#?}");
    }

    #[test]
    fn malformed_pnpm_lockfile_warns_and_obeys_fallback_policy() {
        for (change, warning_type) in [
            ("malformed-pnpm-lock.yaml", "lockfile-pnpm-malformed"),
            (
                "unsupported-pnpm-lock.yaml",
                "lockfile-pnpm-unsupported-schema",
            ),
        ] {
            for fallback in [false, true] {
                let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../fixtures/test-plan/package-manifest-plan/fixture");
                let fixture = materialize_saved_fixture(&source);
                let root = fixture.path().canonicalize().unwrap();
                git_init(&root);
                git_commit_all(&root, "base");
                let lockfile = root.join("pnpm-lock.yaml");
                std::fs::copy(root.join("changes").join(change), &lockfile).unwrap();
                let plan = crate::tests::plan::generate_plan(&plan_args(
                    &root,
                    vec![lockfile],
                    Some("HEAD"),
                    fallback,
                ))
                .unwrap();
                assert_eq!(plan.fallback_triggered, fallback, "{plan:#?}");
                assert!(plan
                    .warnings
                    .iter()
                    .any(|warning| warning.r#type == warning_type));
                if fallback {
                    assert_eq!(plan.selected_tests.len(), 5, "{plan:#?}");
                } else {
                    assert_eq!(group(&plan, "dependencies"), Some(&[][..]), "{plan:#?}");
                }
            }
        }
    }

    #[test]
    fn pnpm_workspace_catalog_override_and_patch_changes_remain_broad() {
        for fallback in [false, true] {
            let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../fixtures/test-plan/package-manifest-plan/fixture");
            let fixture = materialize_saved_fixture(&source);
            let root = fixture.path().canonicalize().unwrap();
            git_init(&root);
            git_commit_all(&root, "base");
            let workspace = root.join("pnpm-workspace.yaml");
            std::fs::copy(root.join("changes/pnpm-workspace-broad.yaml"), &workspace).unwrap();
            let plan = crate::tests::plan::generate_plan(&plan_args(
                &root,
                vec![workspace],
                Some("HEAD"),
                fallback,
            ))
            .unwrap();

            assert_eq!(plan.fallback_triggered, fallback, "{plan:#?}");
            if fallback {
                assert_eq!(plan.selected_tests.len(), 5, "{plan:#?}");
            }
        }
    }
}
