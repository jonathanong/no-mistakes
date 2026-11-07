use super::*;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-lock-written-rows-only/fixture/placeholder-provenance",
    )
}

fn config(policy: &str) -> NoMistakesConfig {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-lock-written-rows-only/configs")
        .join(format!("placeholder-provenance-{policy}.yml"));
    serde_yaml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn generated_relation_markers_never_prove_written_rows_or_unwritten_rows() {
    let findings = check(&fixture(), &config("fail")).unwrap();
    assert!(
        findings
            .iter()
            .all(|finding| finding.rule == RULE_ID
                && finding.target.as_deref() == Some("unanalyzable")),
        "generated marker text is not a relation identity: {findings:?}"
    );
    for lock_line in [5, 10, 20, 30, 40, 50] {
        assert!(
            findings.iter().any(|finding| finding.line == lock_line),
            "dynamic lock/write relations must abstain at lock line {lock_line}: {findings:?}"
        );
    }
    assert!(
        findings.iter().all(|finding| !matches!(
            finding.line,
            15 | 16 | 25 | 26 | 35 | 36 | 45 | 46 | 55 | 56
        )),
        "user-authored marker names must remain valid PostgreSQL identifiers: {findings:?}"
    );
}

#[test]
fn unresolved_relation_provenance_obeys_advisory_and_ignore_policy() {
    for policy in ["advise", "ignore"] {
        let findings = check(&fixture(), &config(policy)).unwrap();
        assert!(findings.is_empty(), "{policy}: {findings:?}");
    }
}

#[test]
fn relation_allow_cannot_match_a_generated_table_lock_marker() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-lock-written-rows-only/fixture/placeholder-table-lock",
    );
    let config: NoMistakesConfig = serde_yaml::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-lock-written-rows-only/configs/placeholder-table-lock.yml"
    )))
    .unwrap();
    let findings = check(&fixture, &config).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].rule, RULE_ID);
    assert_eq!(findings[0].target.as_deref(), Some(CHECKS[1]));
    assert_eq!(findings[0].line, 5);
}
