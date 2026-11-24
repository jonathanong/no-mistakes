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
            changes: None,
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
            direct_test_owner: false,
            format: None,
            json: false,
            include_comment: false,
            include_glob: Vec::new(),
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
