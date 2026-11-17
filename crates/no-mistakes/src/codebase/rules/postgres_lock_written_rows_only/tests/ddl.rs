use super::*;

#[test]
fn accepts_ddl_ordering_and_guard_proof_reads() {
    for case in ["ddl-related", "ddl-guard"] {
        let findings = check(&fixture(case), &config()).unwrap();
        assert!(findings.is_empty(), "{case}: {findings:?}");
    }
}

#[test]
fn keeps_table_lock_reports_without_ordered_transaction_proof() {
    for (case, count) in [("ddl-negative", 16), ("ddl-uncertain", 8)] {
        let findings = check(&fixture(case), &config()).unwrap();
        let locks = findings
            .iter()
            .filter(|finding| finding.target.as_deref() == Some(CHECKS[1]))
            .count();
        assert_eq!(locks, count, "{case}: {findings:?}");
    }
}

#[test]
fn uses_prepared_catalog_relationships_without_suffix_collisions() {
    let mut options = config();
    options.rules[0].options["schemaCatalogPath"] = "schema.json".into();
    let findings = check(&fixture("ddl-catalog"), &options).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 19);
}
