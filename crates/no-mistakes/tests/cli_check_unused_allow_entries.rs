use super::check_json_snapshot::assert_saved_fixture;
use std::path::PathBuf;

#[test]
fn unused_allow_entries_check_json_snapshot() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/check/unused-allow-entries");
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut cases = vec![
        ("query", root, PathBuf::new()),
        (
            "other_lists",
            repository.join("fixtures/rules/unused-allow-other"),
            PathBuf::new(),
        ),
        (
            "forbidden_dependencies",
            repository.join("fixtures/rules/unused-allow-forbidden"),
            PathBuf::new(),
        ),
    ];
    for rule in [
        "csharp-no-async-void-delegate",
        "swift-no-raw-print",
        "swift-viewmodel-main-actor",
        "no-raw-ephemeral-port",
        "no-test-git-sha",
        "no-git-identity-mutation",
        "package-json-required-fields",
    ] {
        cases.push((
            rule,
            repository.join(format!("test-cases/rules/{rule}/fixture/allow-audit")),
            PathBuf::new(),
        ));
    }
    cases.push((
        "test-email-domain-policy",
        repository.join("test-cases/rules/test-email-domain-policy/unit-fixture/allow-audit"),
        PathBuf::new(),
    ));
    assert_saved_fixture("unused_allow_entries", &cases);
}
