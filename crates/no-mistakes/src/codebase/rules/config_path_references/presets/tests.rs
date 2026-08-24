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

#[test]
fn workspace_filters_distinguish_exact_conditional_guards_from_shell_prefixes() {
    let document = serde_yaml::from_str(
        r#"
jobs:
  check:
    steps:
      - run: |
          then pnpm install --filter ./after-then...
          env -i CI=1 pnpm install --filter ./env-options...
          CI=1 pnpm install --filter ./assignment-prefix...
          test -f ./guarded/package.json && pnpm install --filter ./guarded...
          if test -d "./if-guarded"; then env CI=1 pnpm install --filter ./if-guarded...; fi
          test -d ./unconditional; pnpm install --filter ./unconditional...
          test -d ./other && pnpm install --filter ./target...
          pnpm install --filter ./trailing-semicolon...;
          pnpm install --filter ./
          pnpm install --filter packages/missing-prefix
          pnpm install --filter '!./negated'
"#,
    )
    .unwrap();

    let extracted = super::pnpm::workspace_filters(&document);
    let values: Vec<_> = extracted.into_iter().map(|item| item.value).collect();

    assert_eq!(
        values,
        vec![
            "./after-then",
            "./env-options",
            "./assignment-prefix",
            "./unconditional",
            "./target",
            "./trailing-semicolon",
        ]
    );
}
