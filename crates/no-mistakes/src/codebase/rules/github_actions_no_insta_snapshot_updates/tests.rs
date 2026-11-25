use super::{check_with_files, RULE_ID};
use crate::config::v2::load_config;
use std::path::PathBuf;

#[test]
fn check_with_files_applies_inline_and_file_suppressions() {
    let fixture = crate::test_support::rule_fixture_root(RULE_ID);
    let root = fixture.join("suppression");
    let config = load_config(&fixture.parent().unwrap().join("configs/enabled.yml")).unwrap();
    let files: Vec<PathBuf> = [
        root.join(".github/workflows/check.yml"),
        root.join(".github/workflows/regenerate.yml"),
    ]
    .into_iter()
    .collect();
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, ".github/workflows/check.yml");
    assert_eq!(findings[0].line, 16);
}
