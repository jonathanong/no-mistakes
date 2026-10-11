use super::*;

#[test]
fn scalar_switch_label_effects_follow_fail_ignore_and_physical_suppression() {
    let report = check_with_suppressed("postgres-no-offset", "switch-scalar-labels-fail", true);
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 3, "{report}");
    for (finding, (line, target)) in
        findings
            .iter()
            .zip([(5, "unanalyzable"), (10, "unanalyzable"), (16, "offset")])
    {
        assert_eq!(finding["line"], line, "{report}");
        assert_eq!(finding["target"], target, "{report}");
    }
    let suppressed = report["suppressed"].as_array().unwrap();
    assert_eq!(suppressed.len(), 1, "{report}");
    assert_eq!(suppressed[0]["line"], 23, "{report}");

    let ignored = check("postgres-no-offset", "switch-scalar-labels-ignore");
    let findings = ignored["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "{ignored}");
    assert_eq!(findings[0]["line"], 16, "{ignored}");
    assert_eq!(findings[0]["target"], "offset", "{ignored}");
}

#[test]
fn scalar_switch_label_effects_preserve_literal_sql_checks() {
    let report = check("postgres-no-offset", "switch-scalar-labels-good");
    assert!(report["rules"].as_array().unwrap().is_empty(), "{report}");
}
