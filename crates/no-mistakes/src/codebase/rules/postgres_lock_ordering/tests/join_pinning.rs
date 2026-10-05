use super::*;

// A relation pinned through a join key to a single-row relation, and interpolated binds.
#[test]
fn join_keys_and_interpolations_pin_lock_targets() {
    assert!(findings_with_catalog("pass-join-pinned").is_empty());
}

#[test]
fn unproven_joins_and_user_authored_markers_fail_closed() {
    let findings = findings_with_catalog("fail-join-pinned");
    let lines: Vec<_> = findings.iter().map(|finding| finding.line).collect();
    assert_eq!(lines, [7, 18, 29, 40, 52], "{findings:#?}");
    for finding in &findings {
        assert!(finding.message.contains("ABBA"), "{findings:#?}");
    }
}
