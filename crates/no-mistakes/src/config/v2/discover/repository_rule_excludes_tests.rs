use std::path::Path;

#[test]
fn repository_rule_applications_exclude_fixture_trees() {
    // docs/configuration/suppressions.md: the disable audit unions every
    // repository rule's files, and options.include does not shrink that union.
    // This reads the repository config so a new rule without that exclude fails.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let config = super::super::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    for rule in &config.rules {
        for glob in ["fixtures/**", "test-cases/**"] {
            assert!(
                rule.exclude.iter().any(|item| item == glob),
                "{} is missing {glob} in {:?}",
                rule.rule,
                rule.exclude
            );
        }
    }
}
