use crate::config::v2::NoMistakesConfig;
use std::path::PathBuf;

fn fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../test-cases/rules/github-actions-test-timeout-literals/fixture/3304-dispatch",
        ),
    )
}

fn files(root: &std::path::Path) -> Vec<PathBuf> {
    [
        ".github/workflows/ci.test.mts",
        ".github/workflows/ordinary.mts",
        "scripts/timeout.mts",
    ]
    .into_iter()
    .map(|file| root.join(file))
    .collect()
}

fn config(name: &str) -> NoMistakesConfig {
    let path = crate::test_support::rule_fixture_root(super::RULE_ID)
        .parent()
        .unwrap()
        .join("configs")
        .join(name);
    serde_yaml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn default_and_custom_globs_select_only_their_workflow_files() {
    let root = fixture_root();
    let all_files = files(&root);
    let default = config("default.yml");
    let default_findings = super::check_with_files(&root, &default, &all_files).unwrap();
    assert_eq!(default_findings.len(), 1, "{default_findings:?}");
    assert_eq!(default_findings[0].file, ".github/workflows/ci.test.mts");

    let custom = config("3304-custom.yml");
    let custom_findings = super::check_with_files(&root, &custom, &all_files).unwrap();
    assert_eq!(custom_findings.len(), 1, "{custom_findings:?}");
    assert_eq!(custom_findings[0].file, "scripts/timeout.mts");
}

#[test]
fn reasoned_allow_suppresses_match_and_stale_allow_is_reported() {
    let root = fixture_root();
    let all_files = files(&root);
    let allowed = config("3304-allow.yml");
    assert!(super::check_with_files(&root, &allowed, &all_files)
        .unwrap()
        .is_empty());

    let stale = config("3304-stale.yml");
    let findings = super::check_with_files(&root, &stale, &all_files).unwrap();
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert!(findings.iter().any(|finding| finding
        .message
        .contains("duplicates a timeout-minutes value")));
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("stale")));
}
