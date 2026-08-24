#[test]
fn workspace_filters_only_read_executable_yaml_run_scalars() {
    let workflow = include_str!("../../../../../../../fixtures/rules/config-path-references/pnpm-workspace-filters/.github/workflows/filters.yml");
    let document = serde_yaml::from_str(workflow).unwrap();
    let extracted = super::pnpm::workspace_filters(&document);
    let values: Vec<_> = extracted.into_iter().map(|item| item.value).collect();
    assert_eq!(
        values,
        vec![
            "./env-wrapper",
            "./command-wrapper",
            "./unconditional",
            "./optional",
            "./packages/app",
            "./src/*",
            "./multiline"
        ]
    );
    assert!(super::matches_preset(
        "pnpm-workspace-filters",
        "action.yml",
        ".github/actions/setup/action.yml"
    ));
}

#[test]
fn workspace_filters_scope_guards_and_accept_continued_selectors() {
    let document = serde_yaml::from_str(
        r#"
jobs:
  check:
    steps:
      - run: |
          if [ -d ./packages/app ]; then pnpm install --filter ./packages/app...; fi
          pnpm install --filter \
            {./packages/app}...
          pnpm install --filter './src/*...'
"#,
    )
    .unwrap();
    let extracted = super::pnpm::workspace_filters(&document);
    let values: Vec<_> = extracted.into_iter().map(|item| item.value).collect();
    assert_eq!(values, vec!["./packages/app", "./src/*"]);
}
