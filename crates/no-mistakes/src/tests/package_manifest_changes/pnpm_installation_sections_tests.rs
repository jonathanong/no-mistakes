use crate::test_support::{git_commit_all, git_init, materialize_saved_fixture};
use crate::tests::{PlanArgs, TestFramework};
use std::path::{Path, PathBuf};

#[test]
fn unmodeled_pnpm_installation_sections_warn_and_obey_fallback_policy_across_v5_to_v9() {
    for change in [
        "unmodeled-pnpm-v5.yaml",
        "unmodeled-pnpm-v6.yaml",
        "unmodeled-pnpm-v7.yaml",
        "unmodeled-pnpm-v8.yaml",
        "unmodeled-pnpm-v9.yaml",
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

            let plan =
                crate::tests::plan::generate_plan(&plan_args(&root, lockfile, fallback)).unwrap();
            assert_eq!(plan.fallback_triggered, fallback, "{change}: {plan:#?}");
            assert!(
                plan.warnings.iter().any(|warning| {
                    warning.r#type == "lockfile-pnpm-unmodeled-installation-section"
                }),
                "{change}: {plan:#?}"
            );
        }
    }
}

fn plan_args(root: &Path, changed_file: PathBuf, fallback: bool) -> PlanArgs {
    PlanArgs {
        framework: Some(TestFramework::Vitest),
        root: root.to_path_buf(),
        config: None,
        tsconfig: None,
        base: Some("HEAD".to_string()),
        head: None,
        from_git_diff: None,
        changed_file: vec![changed_file],
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
