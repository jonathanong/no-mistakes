use super::*;

#[test]
fn configured_globs_seed_only_actual_visible_runner_parents() {
    let root = Path::new("/repo");
    let visible =
        ["packages/b/vitest.config.ts", "packages/a/vitest.config.ts"].map(|path| root.join(path));
    let mut config = NoMistakesConfig::default();
    assert!(configured_runner_config_dirs_from_visible(root, &config, &visible).is_empty());
    config.tests.vitest.configs = Some(StringOrList::Many(vec![
        "packages/*/vitest.config.ts".into(),
        "packages/*/vitest.config.ts".into(),
        "ignored/*/vitest.config.ts".into(),
    ]));
    config.tests.playwright.configs = Some(StringOrList::One("e2e/playwright.config.ts".into()));
    assert_eq!(
        configured_runner_config_dirs_from_visible(root, &config, &visible),
        vec![
            root.join("packages/a"),
            root.join("packages/b"),
            root.join("e2e"),
        ]
    );
    config.tests.vitest.configs = Some(StringOrList::One("/".into()));
    config.tests.playwright.configs = None;
    assert!(configured_runner_config_dirs_from_visible(root, &config, &visible).is_empty());
}
