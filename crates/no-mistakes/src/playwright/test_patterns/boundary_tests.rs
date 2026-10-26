use crate::playwright::playwright_config::load_many;
use crate::playwright::test_support::fixture_path;

#[test]
fn playwright_pattern_unsupported_entries_have_property_locations() {
    let root = fixture_path(&["playwright-configs", "regexp-boundaries"]);
    let config = load_many(&root, &[root.join("playwright.config.ts")], None).unwrap();
    for (index, project) in config.projects.iter().enumerate() {
        let expected_unknown = (3..=10).contains(&index);
        assert_eq!(
            !project.pattern_options.test_match.warnings.is_empty(),
            expected_unknown,
            "{:?}",
            project.name
        );
        for warning in &project.pattern_options.test_match.warnings {
            assert!(warning
                .render(&root)
                .starts_with(&format!("playwright.config.ts:{}:", index + 7)));
        }
    }
    // A non-u JS /i regex folds ASCII letters, not Kelvin-sign/long-S characters.
    let regex = &config.projects[0].pattern_options.test_match.regexes[0];
    assert!(regex.is_match("/tests/EXAMPLE.e2e.ts"));
    assert!(!regex.is_match("/tests/K.e2e.ts"));
    assert!(!regex.is_match("/tests/ſ.e2e.ts"));
    assert!(!regex.is_match("/tests/example.e2e.ts\n"));
}
