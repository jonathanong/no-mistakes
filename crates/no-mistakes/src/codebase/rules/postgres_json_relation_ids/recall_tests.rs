use super::tests::{run, IDS};

#[test]
fn case_and_coalesce_preserve_top_level_json_keys_and_same_row_fk_writes() {
    let findings = run(IDS, &["db/recall-wrappers-fail.sql"]);
    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert!(findings
        .iter()
        .all(|finding| finding.target.as_deref() == Some("column:orders.details")));
    assert!(run(IDS, &["db/recall-wrappers-pass.sql"]).is_empty());
}

#[test]
fn wildcard_projections_do_not_hide_known_json_expressions() {
    let findings = run(IDS, &["db/recall-wildcards.sql"]);
    assert_eq!(findings.len(), 3, "{findings:#?}");
    assert!(findings.iter().all(|finding| {
        finding.message.contains("key \"accountId\"")
            && finding.message.contains("orders")
            && matches!(
                finding.target.as_deref(),
                Some("column:orders.details" | "table:orders")
            )
    }));
    assert!(findings
        .iter()
        .any(|finding| finding.target.as_deref() == Some("table:orders")));
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [3, 5, 8]
    );
}

#[test]
fn conflict_update_arms_require_their_own_mapped_fk_write() {
    let findings = run(IDS, &["db/recall-conflict-fail.sql"]);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].line, 4);
    assert!(run(IDS, &["db/recall-conflict-pass.sql"]).is_empty());
}

#[test]
fn recovered_prefixes_of_opaque_embedded_sql_remain_outside_the_rule() {
    assert!(run(IDS, &["src/recall-opaque-tail.ts"]).is_empty());
}

#[test]
fn jsonb_constructor_assignment_to_text_still_exposes_the_reference() {
    let findings = run(IDS, &["db/recall-constructor-text.sql"]);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].target.as_deref(), Some("column:orders.note"));
}

#[test]
fn missing_catalog_ordinals_cannot_excuse_a_positional_json_reference() {
    let options = "schemaCatalogPath: schema-missing-ordinals.json\nidKeyPattern: Id$\nkeyColumnMap: {accountId: account_id}\nsqlInclude: ['db/*.sql']";
    let findings = run(options, &["db/positional-missing-ordinals.sql"]);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].line, 2);
    assert_eq!(findings[0].target.as_deref(), Some("table:orders"));
}
