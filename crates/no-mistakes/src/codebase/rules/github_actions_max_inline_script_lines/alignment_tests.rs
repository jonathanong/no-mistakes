use super::*;

#[test]
fn unsupported_scripts_preserve_other_findings_and_exact_source_locations() {
    let root = crate::test_support::rule_fixture_root(RULE_ID).join("alignment");
    let config_path = root
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("configs/three.yml");
    let config = serde_yaml::from_str(&std::fs::read_to_string(config_path).unwrap()).unwrap();
    let findings = check(&root, &config).unwrap();
    let oversized: Vec<_> = findings
        .iter()
        .filter(|f| f.message.starts_with("Inline script"))
        .collect();
    assert_eq!(oversized.len(), 17);
    assert_eq!(findings.len(), 29);
    for finding in oversized {
        let source = std::fs::read_to_string(root.join(&finding.file)).unwrap();
        let line = source.lines().nth(finding.line - 1).unwrap().trim();
        assert!(
            line == "- run: |" || line == "script: |",
            "{}:{}: {line}",
            finding.file,
            finding.line
        );
        assert!(finding.message.contains("4 non-blank lines"));
    }
}
