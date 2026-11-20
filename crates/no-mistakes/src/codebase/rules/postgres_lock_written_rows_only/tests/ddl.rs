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
    for (case, count) in [("ddl-negative", 18), ("ddl-uncertain", 13)] {
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

#[test]
fn parsed_detach_partition_is_complete_for_later_policies() {
    let path = fixture("ddl-related").join("src/query.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let config = config();
    let compiled = compile(&config.rules[0].try_rule_options().unwrap()).unwrap();
    let embedded = crate::codebase::postgres::extract_embedded_sql_from_source(
        &path,
        &source,
        &compiled.embedded,
    );
    let call = embedded
        .calls
        .iter()
        .find(|call| {
            call.sql_text
                .as_deref()
                .is_some_and(|sql| sql.starts_with("ALTER TABLE events DETACH"))
        })
        .unwrap();
    let facts =
        crate::codebase::postgres::extract_sql_statement_facts_for_embedded_call(call).unwrap();
    assert!(!facts.parse_failed);
    assert!(
        facts.parse_failures.is_empty(),
        "{:?}",
        facts.parse_failures
    );
}

#[test]
fn generated_ddl_identifiers_do_not_prove_relationships() {
    let root = fixture("ddl-dynamic");
    let config: NoMistakesConfig =
        serde_yaml::from_slice(&std::fs::read(root.join("../../configs/ddl-dynamic.yml")).unwrap())
            .unwrap();
    let findings = check(&root, &config).unwrap();
    assert_eq!(findings.len(), 3, "{findings:?}");
    assert!(findings
        .iter()
        .all(|finding| finding.target.as_deref() == Some(CHECKS[1])));
}
