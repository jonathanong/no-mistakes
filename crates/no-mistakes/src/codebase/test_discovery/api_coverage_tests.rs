use super::*;
use crate::config::v2::schema::StringOrList;

#[test]
fn visible_api_globs_follow_the_supplied_test_inventory() {
    let root = fixture_root("test-discovery-vitest-defaults");
    let mut config = NoMistakesConfig::default();
    config.tests.vitest.configs = Some(StringOrList::One("vitest.config.mts".to_string()));

    let visible = crate::codebase::ts_source::discover_visible_paths(&root);
    let selected: Vec<_> = visible
        .into_iter()
        .filter(|path| path.ends_with("src/default.test.ts"))
        .collect();
    let tsconfig = resolve_tsconfig_lossy(&root, &selected);

    assert_eq!(
        discovered_test_globs_from_visible(
            &root,
            &config,
            TestRunner::Vitest,
            &selected,
            &tsconfig,
        )
        .unwrap(),
        Some(vec!["src/default.test.ts".to_string()])
    );
    assert_eq!(
        discovered_test_globs_from_visible(&root, &config, TestRunner::Vitest, &[], &tsconfig,)
            .unwrap(),
        None
    );
}
