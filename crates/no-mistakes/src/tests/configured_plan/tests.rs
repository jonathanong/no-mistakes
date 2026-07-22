use super::*;
use no_mistakes::config::v2::schema::{
    NoMistakesConfig, Project, StringOrList, TestPlanIgnoredChangedTestsFramework,
    TestPlanProjectDependency, TestPlanTargetedProjectDependency,
};

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
