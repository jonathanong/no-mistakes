use super::*;
use no_mistakes::config::v2::schema::{
    NoMistakesConfig, Project, StringOrList, TestPlanIgnoredChangedTestsFramework,
    TestPlanProjectDependency, TestPlanTargetedProjectDependency,
};

fn vitest_setup_args(root: PathBuf, changed_file: Vec<PathBuf>) -> PlanArgs {
    PlanArgs {
        framework: Some(TestFramework::Vitest),
        root,
        config: None,
        tsconfig: None,
        base: None,
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
        environment: "pre-push".to_string(),
        limit_percent: None,
        limit_files: None,
        global_config_fallback: Some(false),
        format: None,
        json: false,
    }
}

fn vitest_setup_fixture() -> (tempfile::TempDir, PathBuf) {
    let source = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/vitest-setup-dependencies"),
    );
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    (fixture, root)
}

#[test]
fn vitest_setup_transitive_dependency_selects_only_the_owning_project() {
    let (_fixture, root) = vitest_setup_fixture();
    let mut args = vitest_setup_args(root.clone(), vec![root.join("setup/resolved-helper.ts")]);
    args.config = Some(root.join("resolved.no-mistakes.yml"));
    let plan = crate::tests::plan::generate_plan(&args).unwrap();

    assert_eq!(
        plan.selected_tests
            .iter()
            .map(|test| test.test_file.as_str())
            .collect::<Vec<_>>(),
        ["resolved/resolved.test.ts"]
    );
    assert!(!plan.fallback_triggered, "{plan:#?}");
    let reason = &plan.selected_tests[0].reasons[0];
    assert_eq!(reason.via, ["dependency", "vitest-setup"]);
    assert_eq!(
        reason.via_details,
        Some(vec![None, Some("setupFiles".to_string())])
    );
}

#[test]
fn dynamic_vitest_setup_uses_owner_scoped_fallback_without_global_opt_in() {
    let (_fixture, root) = vitest_setup_fixture();
    let plan = crate::tests::plan::generate_plan(&vitest_setup_args(
        root.clone(),
        vec![root.join("inherits/dynamic-only.ts")],
    ))
    .unwrap();

    assert_eq!(
        plan.selected_tests
            .iter()
            .map(|test| test.test_file.as_str())
            .collect::<Vec<_>>(),
        ["inherits/inherited.test.ts"]
    );
    assert!(plan.fallback_triggered, "{plan:#?}");
    assert!(plan
        .fallback_reason
        .as_deref()
        .is_some_and(|reason| reason.contains("owning project")));
    assert!(plan
        .warnings
        .iter()
        .any(|warning| warning.r#type == "vitest-setup-dynamic"));
    assert!(plan
        .warnings
        .iter()
        .any(|warning| warning.r#type == "vitest-setup-unresolved"));
}

#[test]
fn dynamic_vitest_setup_import_helper_outside_owner_scope_uses_owner_fallback() {
    let (_fixture, root) = vitest_setup_fixture();
    let plan = crate::tests::plan::generate_plan(&vitest_setup_args(
        root.clone(),
        vec![root.join("config/setup-selector.ts")],
    ))
    .unwrap();

    assert_eq!(
        plan.selected_tests
            .iter()
            .map(|test| test.test_file.as_str())
            .collect::<Vec<_>>(),
        ["inherits/inherited.test.ts"]
    );
    assert!(plan.fallback_triggered, "{plan:#?}");
    assert!(plan
        .fallback_reason
        .as_deref()
        .is_some_and(|reason| reason.contains("owning project")));
}

#[test]
fn dynamic_vitest_setup_transitive_helper_changes_and_deletions_use_owner_fallback() {
    let (_fixture, root) = vitest_setup_fixture();
    let helper = root.join("config/transitive-dynamic-helper.ts");
    let changed =
        crate::tests::plan::generate_plan(&vitest_setup_args(root.clone(), vec![helper.clone()]))
            .unwrap();

    let mut deleted_args = vitest_setup_args(root.clone(), Vec::new());
    deleted_args.diff_content = Some(
        "diff --git a/config/transitive-dynamic-helper.ts b/config/transitive-dynamic-helper.ts\n\
--- a/config/transitive-dynamic-helper.ts\n\
+++ /dev/null\n\
@@ -1 +0,0 @@\n\
-export const transitiveDynamicSetup = () => process.env.SETUP_FILE\n"
            .to_string(),
    );
    let deleted = crate::tests::plan::generate_plan(&deleted_args).unwrap();

    for plan in [changed, deleted] {
        assert_eq!(
            plan.selected_tests
                .iter()
                .map(|test| test.test_file.as_str())
                .collect::<Vec<_>>(),
            ["closure-owner/closure.test.ts"],
            "{plan:#?}"
        );
        assert!(plan.fallback_triggered, "{plan:#?}");
        assert!(plan
            .fallback_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("owning project")));
    }
}

#[test]
fn dynamic_vitest_setup_with_known_empty_owner_never_widens_to_framework() {
    let (_fixture, root) = vitest_setup_fixture();
    let plan = crate::tests::plan::generate_plan(&vitest_setup_args(
        root.clone(),
        vec![root.join("empty-owner/source.ts")],
    ))
    .unwrap();

    assert!(plan.selected_tests.is_empty(), "{plan:#?}");
    assert!(plan.fallback_triggered, "{plan:#?}");
    assert!(plan
        .fallback_reason
        .as_deref()
        .is_some_and(|reason| reason.contains("owning project")));
    assert!(!plan
        .fallback_reason
        .as_deref()
        .is_some_and(|reason| reason.contains("discovered Vitest tests")));
}

#[test]
fn unresolved_vitest_setup_deleted_candidate_uses_owner_scoped_fallback() {
    let (_fixture, root) = vitest_setup_fixture();
    let mut args = vitest_setup_args(root.clone(), Vec::new());
    args.diff_content = Some(
        "diff --git a/inherits/setup/missing.ts b/inherits/setup/missing.ts\n\
--- a/inherits/setup/missing.ts\n\
+++ /dev/null\n\
@@ -1 +0,0 @@\n\
-export const removed = true\n"
            .to_string(),
    );
    let plan = crate::tests::plan::generate_plan(&args).unwrap();

    assert_eq!(
        plan.selected_tests
            .iter()
            .map(|test| test.test_file.as_str())
            .collect::<Vec<_>>(),
        ["inherits/inherited.test.ts"]
    );
    assert!(plan.fallback_triggered, "{plan:#?}");
    assert!(plan.warnings.iter().any(|warning| {
        warning.r#type == "vitest-setup-unresolved"
            && warning.message.contains("`./setup/missing.ts`")
    }));
}

#[test]
fn unresolved_vitest_setup_deleted_alias_and_base_url_index_candidates_use_owner_fallback() {
    let (_fixture, root) = vitest_setup_fixture();
    let cases = [
        (
            "aliases/setup/missing.tsx",
            "alias-owner/alias-owner.test.ts",
        ),
        (
            "aliases/setup/missing.jsx",
            "alias-owner/alias-owner.test.ts",
        ),
        (
            "base-setup/missing/index.mts",
            "base-owner/base-owner.test.ts",
        ),
    ];
    for (deleted_path, expected_test) in cases {
        let mut args = vitest_setup_args(root.clone(), Vec::new());
        args.diff_content = Some(format!(
            "diff --git a/{deleted_path} b/{deleted_path}\n--- a/{deleted_path}\n+++ /dev/null\n@@ -1 +0,0 @@\n-export const removed = true\n"
        ));
        let plan = crate::tests::plan::generate_plan(&args).unwrap();
        assert_eq!(
            plan.selected_tests
                .iter()
                .map(|test| test.test_file.as_str())
                .collect::<Vec<_>>(),
            [expected_test],
            "{deleted_path}: {plan:#?}"
        );
        assert!(plan.fallback_triggered, "{deleted_path}: {plan:#?}");
    }
}

#[test]
fn dependency_trigger_ignores_changed_test_discovery_errors_for_source_changes() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/test-discovery-policy-fallback/fixture");
    let mut config = NoMistakesConfig::default();
    config.projects.insert(
        "src".to_string(),
        Project {
            root: Some("src".to_string()),
            ..Default::default()
        },
    );
    config.tests.vitest.configs = Some(StringOrList::One("missing.vitest.config.mts".to_string()));
    config
        .test_plan
        .vitest
        .full_suite_triggers
        .ignore_changed_tests = vec![TestPlanIgnoredChangedTestsFramework::Vitest];
    config
        .test_plan
        .vitest
        .full_suite_triggers
        .projects
        .insert("src".to_string(), TestPlanProjectDependency::All(true));

    let plan_args = PlanArgs {
        framework: Some(TestFramework::Vitest),
        root: root.clone(),
        config: None,
        tsconfig: None,
        base: None,
        head: None,
        from_git_diff: None,
        changed_file: vec![root.join("src/component.mts")],
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
        global_config_fallback: None,
        format: None,
        json: false,
    };
    let prepared = crate::tests::prepared_plan::PreparedTestPlanRequest::prepare(&plan_args)
        .expect("fixture request should prepare");

    let trigger = dependency_triggers(
        &root,
        &config,
        TestFramework::Vitest,
        &[root.join("src/component.mts")],
        &prepared,
    )
    .unwrap();

    assert!(trigger.fallback.is_some());
}

#[test]
fn dependency_patterns_use_ordered_negation_and_reinclusion() {
    let project = Project {
        root: Some("src".to_string()),
        ..Project::default()
    };
    let trigger = TestPlanProjectDependency::Targeted(TestPlanTargetedProjectDependency {
        paths: vec![
            "**/*.ts".to_string(),
            "!generated/**".to_string(),
            "generated/keep.ts".to_string(),
        ],
        targets: vec!["unit".to_string()],
    });
    let patterns = super::dep_triggers::project_dependency_patterns("src", &project, &trigger);
    assert_eq!(
        patterns,
        vec!["src/**/*.ts", "!src/generated/**", "src/generated/keep.ts"]
    );
    let patterns = super::dep_triggers::compile_ordered_patterns(&patterns).unwrap();
    assert!(super::dep_triggers::matches_ordered(
        &patterns,
        "src/generated/keep.ts"
    ));
    assert!(!super::dep_triggers::matches_ordered(
        &patterns,
        "src/generated/drop.ts"
    ));
    assert!(super::dep_triggers::compile_ordered_patterns(&["[".to_string()]).is_err());
}

#[test]
fn explicit_ignored_changed_sources_impact_visible_tests_without_ignored_shadows() {
    let fixture = crate::test_support::materialize_gitignore_fixture("prepared-tsconfig");
    let root = no_mistakes::codebase::ts_resolver::normalize_path(fixture.path());
    let diff = "diff --git a/ignored-explicit/Button.tsx b/ignored-explicit/Button.tsx\n\
                --- a/ignored-explicit/Button.tsx\n\
                +++ b/ignored-explicit/Button.tsx\n\
                @@ -1,1 +1,1 @@\n\
                -export function IgnoredButton() {\n\
                +export function IgnoredButton() {\n";
    let inputs = [
        (vec![PathBuf::from("ignored-explicit/Button.tsx")], None),
        (Vec::new(), Some(diff.to_string())),
    ];

    for (changed_file, diff_content) in inputs {
        let plan = crate::tests::plan::generate_plan(&PlanArgs {
            framework: None,
            root: root.clone(),
            config: None,
            tsconfig: None,
            base: None,
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
            diff_content,
            environment: "pre-push".to_string(),
            limit_percent: None,
            limit_files: None,
            global_config_fallback: None,
            format: None,
            json: false,
        })
        .unwrap();
        let selected = plan
            .selected_tests
            .iter()
            .map(|test| test.test_file.as_str())
            .collect::<Vec<_>>();

        assert!(
            selected.contains(&"tests/ignored-button.test.tsx"),
            "{selected:#?}"
        );
        assert!(
            !selected.contains(&"ignored-transitive/Button.test.tsx"),
            "{selected:#?}"
        );
    }
}
