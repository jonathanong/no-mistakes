use super::{check_with_files_sources_and_facts, config, fixture, RULE_ID};
use std::sync::Arc;

#[test]
fn opaque_remainders_keep_definite_prefix_violations_in_every_mode() {
    let root = fixture();
    let file = root.join("tests/prefix-policy.test.ts");
    let files = vec![file];
    let sources = super::super::super::source_store_for_files(&files);
    for mode in ["fail", "advise", "ignore"] {
        let mut config = config(false);
        config.rules[0].options["unanalyzableSql"] = mode.into();
        let facts = crate::codebase::postgres::prepare_rule_sql_facts(
            &root,
            &files,
            Arc::clone(&sources),
            &config,
            &[RULE_ID],
        )
        .unwrap()
        .with_fresh_sql_verdicts();
        let findings =
            check_with_files_sources_and_facts(&root, &config, &files, &sources, &facts).unwrap();
        for target in ["CREATE TABLE", "session_replication_role", "pg_sleep"] {
            assert!(
                findings
                    .iter()
                    .any(|finding| finding.target.as_deref() == Some(target)),
                "{mode}: {findings:?}"
            );
        }
        assert_eq!(
            findings
                .iter()
                .any(|finding| finding.target.as_deref() == Some("unanalyzable-sql")),
            mode == "fail",
            "{mode}: {findings:?}"
        );
        let verdicts = facts.sql_verdicts();
        assert!(verdicts.iter().any(|verdict| verdict.advisory) == (mode == "advise"));
    }
}
