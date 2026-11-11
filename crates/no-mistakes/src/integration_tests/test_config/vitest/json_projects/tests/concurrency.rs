use super::*;

#[test]
fn saved_json_sequence_values_keep_unknown_separate_from_absent_and_false() {
    let root =
        crate::test_support::rule_fixture_root("test-no-live-repository-scan/facts/coverage");
    let source = std::fs::read_to_string(root.join("config-sequence.json")).unwrap();
    let projects = parse_json(&source).unwrap();
    assert_eq!(
        projects
            .iter()
            .map(|project| (
                project.policy_name.as_deref().unwrap(),
                project.vitest_concurrent
            ))
            .collect::<Vec<_>>(),
        [
            ("enabled", true),
            ("disabled", false),
            ("unknown value", true),
            ("unknown object", true),
            ("other key", false),
            ("absent", false)
        ]
    );
}
