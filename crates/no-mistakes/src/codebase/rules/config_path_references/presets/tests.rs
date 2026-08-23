#[test]
fn workspace_filters_handle_braces_wildcards_and_guards() {
    let extracted = super::pnpm::workspace_filters(
        r#"
pnpm install --filter '{./packages/app}...' \
  --filter "./src/*..."
if [ -d "./optional" ]; then pnpm install --filter './optional...'; fi
test -f ./another/package.json && pnpm install --filter './another...'
"#,
    );
    let values: Vec<_> = extracted.into_iter().map(|item| item.value).collect();
    assert_eq!(values, vec!["./packages/app", "./src/*"]);
    assert!(super::matches_preset(
        "pnpm-workspace-filters",
        "action.yml",
        ".github/actions/setup/action.yml"
    ));
}
