use super::*;

fn fixture(name: &str) -> PathBuf {
    crate::test_support::rule_fixture_root(RULE_ID).join(name)
}

fn config(name: &str) -> NoMistakesConfig {
    let path = crate::test_support::rule_fixture_root(RULE_ID)
        .parent()
        .unwrap()
        .join(format!("configs/{name}.yml"));
    serde_yaml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn standalone_discovers_workflows_and_composites() {
    let findings = check(&fixture("boundary"), &config("three")).unwrap();
    assert_eq!(findings.len(), 5);
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.file.contains("/actions/"))
            .count(),
        2
    );
}

#[test]
fn standalone_honors_yaml_script_key_suppressions() {
    assert!(check(&fixture("suppression"), &config("three"))
        .unwrap()
        .is_empty());
}

#[test]
fn standalone_reports_unreadable_workflow_sources() {
    let findings = check(&fixture("unreadable"), &config("three")).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, ".github/workflows/broken.yml");
    assert_eq!(findings[0].line, 2);
    assert!(findings[0].message.contains("UTF-8"));
}

#[test]
fn standalone_rejects_nonpositive_limits() {
    let error = check(&fixture("boundary"), &config("invalid")).unwrap_err();
    assert!(error
        .to_string()
        .contains("maxLines must be a positive integer"));
}

#[test]
fn scalar_delimiters_and_yaml_comments_do_not_count() {
    for (name, expected) in [("quoted-scalars", 4), ("block-comments", 11)] {
        let findings = check(&fixture(name), &config("three")).unwrap();
        assert_eq!(findings.len(), expected, "{name}: {findings:?}");
        for finding in findings {
            assert!(finding.message.contains("4 non-blank"), "{finding:?}");
        }
    }
}
