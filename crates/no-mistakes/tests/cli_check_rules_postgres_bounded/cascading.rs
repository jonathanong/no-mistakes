use super::{check_in, findings_of, fixture_dir};

#[test]
fn cascading_descendant_walks_report_one_finding_per_statement() {
    let root = fixture_dir("variants/cascading-statements");
    let findings = findings_of(check_in(&root, ".no-mistakes.yml"));
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert!(findings
        .iter()
        .all(|(_, _, target)| target == "table:accounts, table:orders"));
}

#[test]
fn aggregated_statement_roots_preserve_allow_usage_and_relation_suppressions() {
    let root = fixture_dir("variants/cascading-statements");
    let allowed = findings_of(check_in(&root, "allow.yml"));
    assert_eq!(allowed.len(), 2, "{allowed:?}");
    assert!(allowed
        .iter()
        .all(|(_, _, target)| target == "table:accounts"));
    let mut suppressed = findings_of(check_in(&root, "suppression.yml"));
    suppressed.sort();
    assert_eq!(suppressed.len(), 3, "{suppressed:?}");
    assert_eq!(suppressed[0].2, "table:accounts");
    assert_eq!(suppressed[1].2, "table:accounts, table:orders");
    assert_eq!(suppressed[2].2, "table:invoices");
}

#[test]
fn alternatives_share_one_statement_finding_with_every_unbounded_root() {
    let root = fixture_dir("variants/cascading-statements");
    let findings = findings_of(check_in(&root, "variants.yml"));
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(
        findings[0].2,
        "table:accounts, table:invoices, table:orders"
    );
}

#[test]
fn cascading_statement_check_json_snapshot() {
    let root = fixture_dir("variants/cascading-statements");
    crate::check_json_snapshot::assert_saved_fixture(
        "postgres_bounded_cascading_statements",
        &[
            ("roots", root.clone(), root.join(".no-mistakes.yml")),
            ("allow", root.clone(), root.join("allow.yml")),
            ("variants", root.clone(), root.join("variants.yml")),
            ("suppression", root.clone(), root.join("suppression.yml")),
        ],
    );
}
