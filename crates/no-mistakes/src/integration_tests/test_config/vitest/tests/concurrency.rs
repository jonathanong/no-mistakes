use super::*;

#[test]
fn saved_sequence_objects_preserve_unknown_and_known_sequential_projects() {
    let root =
        crate::test_support::rule_fixture_root("test-no-live-repository-scan/facts/coverage");
    let path = root.join("config-sequence.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let projects = crate::integration_tests::test_support::parse_vitest(
        &source,
        &path,
        &root,
        &root,
        &TsConfig::default(),
    )
    .unwrap();
    assert_eq!(
        projects
            .iter()
            .map(|project| (
                project.policy_name.as_deref().unwrap(),
                project.vitest_concurrent
            ))
            .collect::<Vec<_>>(),
        [
            ("unknown object", true),
            ("spread object", true),
            ("computed key", true),
            ("unrelated key", false),
            ("known false", false)
        ]
    );
}
