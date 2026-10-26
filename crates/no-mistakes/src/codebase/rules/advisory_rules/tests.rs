use super::*;

fn finding(rule: &str) -> RuleFinding {
    RuleFinding {
        rule: rule.into(),
        file: "query.sql".into(),
        line: 1,
        message: "finding".into(),
        import: None,
        target: None,
    }
}

#[test]
fn partitions_advisories_and_preserves_equal_findings_in_both_channels() {
    let json_relation = finding(super::super::POSTGRES_JSON_RELATION_IDS);
    let ordinary = finding("ordinary-rule");
    let diagnostics = AnalysisDiagnostics {
        findings: vec![ordinary.clone(), json_relation.clone(), ordinary.clone()],
        advisories: vec![finding("existing-advisory"), finding("existing-advisory")],
        ..Default::default()
    };

    let actual = partition_findings(diagnostics);

    assert_eq!(actual.findings, vec![ordinary.clone(), ordinary]);
    let mut expected_advisories = vec![
        json_relation,
        finding("existing-advisory"),
        finding("existing-advisory"),
    ];
    expected_advisories.sort();
    assert_eq!(actual.advisories, expected_advisories);
}
