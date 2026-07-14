use super::*;
use crate::config::v2::schema::StringOrList;

fn fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/test-no-unmocked-dynamic-imports/fixture"),
    )
}

#[test]
fn prepared_config_globs_only_expand_aggregate_candidates() {
    let root = fixture();
    let selected_config = root.join("jest.config.cjs");
    let mut config = NoMistakesConfig::default();
    config.tests.jest.configs = Some(StringOrList::One("jest.config.*".to_string()));

    // `jest.config.mjs` also exists and matches the configured glob. Omitting it
    // from the aggregate candidates must keep its matcher and setup files out.
    let prepared = prepare_from_visible(&root, &config, &[selected_config]).unwrap();

    assert!(prepared
        .test_filter()
        .is_match("tests/example.cjs-spec.mts"));
    assert!(!prepared.test_filter().is_match("tests/example.spec.mts"));
    assert_eq!(prepared.setup_data().len(), 1);
    assert!(prepared.setup_data()[0].setup_files.is_empty());
}

#[test]
fn aggregate_rule_uses_prepared_config_without_standalone_discovery() {
    let source = include_str!("../with_facts.rs");

    assert!(
        source.contains("config::prepare_from_visible(root, config, shared.graph_file_universe())")
    );
    assert!(!source.contains("config::test_filter("));
    assert!(!source.contains("config::precompute_setup_data("));
    assert!(!source.contains("discover_files("));
}

#[test]
fn pass4b_prepared_setup_files_drop_ignored_candidate_and_keep_visible_fallback() {
    let fixture = crate::test_support::materialize_gitignore_fixture("pass4b-shadow");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let visible = crate::codebase::ts_source::discover_visible_paths(&root);
    let mut config = NoMistakesConfig::default();
    config.tests.vitest.configs = Some(StringOrList::One("dynamic/vitest.config.ts".to_string()));

    let prepared = prepare_from_visible(&root, &config, &visible).unwrap();

    assert_eq!(
        prepared.setup_data()[0].setup_files,
        vec![root.join("dynamic/setup.ts")]
    );
}
