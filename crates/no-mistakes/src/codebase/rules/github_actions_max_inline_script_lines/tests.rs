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
    for (name, expected) in [("quoted-scalars", 6), ("block-comments", 11)] {
        let findings = check(&fixture(name), &config("three")).unwrap();
        assert_eq!(findings.len(), expected, "{name}: {findings:?}");
        for finding in findings {
            assert!(finding.message.contains("4 non-blank"), "{finding:?}");
        }
    }
}

#[test]
fn quoted_scalar_comments_preserve_the_following_script_locations() {
    let root = fixture("quoted-scalars");
    let findings = check(&root, &config("three")).unwrap();
    let source = std::fs::read_to_string(root.join(".github/workflows/check.yml")).unwrap();
    let expected: Vec<_> = source
        .lines()
        .enumerate()
        .filter(|(_, line)| line.trim() == "- run: |")
        .map(|(index, _)| index + 1)
        .collect();
    assert_eq!(expected.len(), 2);
    for line in expected {
        assert!(findings.iter().any(|finding| {
            finding.line == line && finding.message.starts_with("Inline script has 4 non-blank")
        }));
    }
}

#[test]
fn flow_step_locations_honor_suppressions_without_losing_neighbors() {
    let root = fixture("flow-locations");
    let findings = check(&root, &config("three")).unwrap();
    assert_eq!(findings.len(), 7, "{findings:?}");
    assert_eq!(
        findings
            .iter()
            .filter(|f| f.message.starts_with("Script body"))
            .count(),
        4
    );
    for finding in findings {
        assert!(!finding.file.contains("file-suppressed"));
        let source = std::fs::read_to_string(root.join(&finding.file)).unwrap();
        let line = source.lines().nth(finding.line - 1).unwrap().trim();
        if finding.message.starts_with("Script body") {
            assert!(line.starts_with("- {"), "{finding:?}: {line}");
            assert!(!line.contains("no-mistakes-disable"));
        } else {
            assert!(
                line == "- run: |" || line == "run: |",
                "{finding:?}: {line}"
            );
            assert!(finding.message.contains("4 non-blank lines"));
        }
    }
}

#[test]
fn block_action_references_and_anchored_bodies_keep_exact_boundaries() {
    for (name, count) in [("block-uses", 4), ("anchored-scalars", 2)] {
        let root = fixture(name);
        let findings = check(&root, &config("three")).unwrap();
        assert_eq!(findings.len(), count, "{name}: {findings:?}");
        for finding in findings {
            assert!(
                finding.message.starts_with("Inline script has 4 non-blank"),
                "{finding:?}"
            );
            let source = std::fs::read_to_string(root.join(&finding.file)).unwrap();
            let line = source.lines().nth(finding.line - 1).unwrap().trim();
            assert!(
                line.starts_with("script:")
                    || line.starts_with("- run:")
                    || line.starts_with("run:"),
                "{finding:?}: {line}"
            );
        }
    }
}
