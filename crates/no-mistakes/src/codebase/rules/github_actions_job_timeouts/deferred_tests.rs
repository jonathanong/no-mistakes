use super::*;

#[test]
fn deferred_suppression_preserves_used_findings_without_inventing_stale_findings() {
    for (name, expected) in [("deferred-used", 1), ("deferred-stale", 0)] {
        let root = super::tests::fixture(name);
        let files = vec![root.join(".github/workflows/ci.yml")];
        let sources = super::super::source_store_for_files(&files);
        let config = crate::config::v2::NoMistakesConfig {
            rules: vec![crate::config::v2::schema::RuleDef {
                rule: RULE_ID.into(),
                scope: Some(crate::config::v2::schema::RuleScope::Repository),
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(
            check_with_files_and_sources(&root, &config, &files, &sources)
                .unwrap()
                .is_empty()
        );
        let raw =
            check_with_files_and_sources_deferred(&root, &config, &files, &sources, true).unwrap();
        assert_eq!(raw.len(), expected, "{name}: {raw:?}");
    }
}
