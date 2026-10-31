use super::*;

#[test]
fn mocked_module_owner_suppresses_only_proven_chains() {
    let base = crate::test_support::rule_fixture_root("forbidden-calls");
    let fixture = base.parent().unwrap().join("mocked-owner/fixture");
    let findings = run_check(&fixture, None, None).unwrap();
    let files = findings
        .iter()
        .map(|finding| finding.file.replace('\\', "/"))
        .collect::<Vec<_>>()
        .join("\n");
    for kept in [
        "other-through.mts",
        "avoid-through.mts",
        "direct-mock.test.mts",
    ] {
        assert!(files.contains(kept), "{kept} missing from {files}");
    }
    for dropped in [
        "through-owner.mts",
        "barrel-through.mts",
        "nested/through.mts",
        "external-through.mts",
        "jest-through.mts",
    ] {
        assert!(
            !files.contains(dropped),
            "{dropped} still reported in {files}"
        );
    }
}
