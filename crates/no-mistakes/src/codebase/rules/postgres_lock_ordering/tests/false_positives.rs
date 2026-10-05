use super::*;
use crate::codebase::rules::RuleFinding;

fn lines(findings: &[RuleFinding]) -> Vec<usize> {
    findings.iter().map(|finding| finding.line).collect()
}

// #1542: an IN / ANY on a non-key column does not widen a pinned unique-key lookup.
#[test]
fn pinned_unique_key_ignores_in_and_any_on_other_columns() {
    assert!(findings_with_catalog("pass-unique-key-filter").is_empty());
}

#[test]
fn pins_that_do_not_cover_a_unique_key_still_fail_closed() {
    let findings = findings_with_catalog("fail-unique-key-partial");
    assert_eq!(
        lines(&findings),
        [6, 11, 16, 21, 30, 39, 48],
        "{findings:#?}"
    );
    for finding in &findings {
        assert_eq!(finding.target.as_deref(), Some(LOCK_ORDERING_TARGET));
        assert!(finding.message.contains("ABBA"), "{findings:#?}");
    }
}

#[test]
fn pinned_key_without_a_catalog_still_fails_closed() {
    let findings = findings_for("fail-unique-key-no-catalog");
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert!(findings[0].message.contains("ABBA"), "{findings:#?}");
}

// #1543: a derived relation that is not a lock target is not in doubt.
#[test]
fn lateral_join_beside_one_locked_base_table_is_checked() {
    assert!(findings_with_catalog("pass-catalog-lateral").is_empty());
}

#[test]
fn lateral_join_still_fails_when_it_may_be_locked_or_misordered() {
    let findings = findings_with_catalog("fail-catalog-lateral");
    assert_eq!(lines(&findings), [6, 18, 30], "{findings:#?}");
    for finding in &findings {
        assert!(finding.message.contains("schema-catalog"), "{findings:#?}");
    }
}

#[test]
fn interpolated_relation_gets_a_distinct_suppressible_diagnostic() {
    let findings = findings_with_catalog("fail-catalog-interpolated");
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(
        findings[0].target.as_deref(),
        Some(UNRESOLVED_RELATION_TARGET)
    );
    assert!(
        findings[0].message.contains("interpolated"),
        "{findings:#?}"
    );
    assert!(
        findings[0].message.contains("deadlock-safe"),
        "{findings:#?}"
    );
    assert!(
        !findings[0].message.contains("unique-key order"),
        "{findings:#?}"
    );
}
