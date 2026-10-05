use super::*;

#[test]
fn multi_target_of_lists_parse_and_pass_when_ordered() {
    assert!(findings_for("pass-multi-target").is_empty());
}

#[test]
fn multi_target_of_lists_are_checked_like_the_split_form() {
    let findings = findings_for("fail-multi-target");
    assert_eq!(findings.len(), 2, "{findings:#?}");
    for finding in &findings {
        assert_eq!(finding.target.as_deref(), Some(LOCK_ORDERING_TARGET));
        assert!(finding.message.contains("ABBA"), "{findings:#?}");
    }
}

#[test]
fn multi_target_of_lists_require_a_catalog_prefix_for_every_relation() {
    assert!(findings_with_catalog("pass-catalog-multi-target").is_empty());
    let findings = findings_with_catalog("fail-catalog-multi-target");
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert!(findings[0].message.contains("schema-catalog"));
}

#[test]
fn unresolved_name_in_a_multi_target_of_list_fails_closed() {
    let findings = findings_with_catalog("fail-catalog-multi-target-unresolved");
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].target.as_deref(), Some(LOCK_ORDERING_TARGET));
    assert!(findings[0].message.contains("schema-catalog"));
}
