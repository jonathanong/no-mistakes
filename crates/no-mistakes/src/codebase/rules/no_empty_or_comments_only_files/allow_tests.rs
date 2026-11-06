use super::*;

fn fixture() -> PathBuf {
    crate::test_support::rule_fixture_root(RULE_ID).join("allow-audit")
}

#[test]
fn audits_only_readable_selected_supported_entries() {
    let root = fixture();
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let findings = check(&root, &config).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].rule, "unused-allow-entry");
    assert_eq!(findings[0].target.as_deref(), Some("clean.ts"));
}

#[test]
fn partial_failed_and_overlapping_scopes_remain_silent() {
    let root = fixture();
    for name in ["partial.yml", "failed.yml", "overlap.yml"] {
        let config = crate::config::v2::load_v2_config(&root, Some(&root.join(name))).unwrap();
        assert!(check(&root, &config).unwrap().is_empty(), "{name}");
    }
}

#[test]
fn partial_invocation_does_not_infer_absent_entries_are_stale() {
    let root = fixture();
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let files = vec![root.join("used.ts")];
    assert!(check_with_files(&root, &config, &files).unwrap().is_empty());
}
