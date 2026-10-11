use super::*;

#[test]
fn scalar_switch_label_effects_follow_fail_ignore_and_physical_suppression() {
    let report = check_with_suppressed("postgres-no-offset", "switch-scalar-labels-fail", true);
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 8, "{report}");
    for (finding, (line, target)) in findings.iter().zip([
        (5, "unanalyzable"),
        (10, "unanalyzable"),
        (16, "offset"),
        (32, "unanalyzable"),
        (40, "unanalyzable"),
        (46, "unanalyzable"),
        (54, "offset#2"),
        (56, "offset#3"),
    ]) {
        assert_eq!(finding["line"], line, "{report}");
        assert_eq!(finding["target"], target, "{report}");
    }
    let suppressed = report["suppressed"].as_array().unwrap();
    assert_eq!(suppressed.len(), 1, "{report}");
    assert_eq!(suppressed[0]["line"], 23, "{report}");

    let ignored = check("postgres-no-offset", "switch-scalar-labels-ignore");
    let findings = ignored["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 3, "{ignored}");
    for (finding, (line, target)) in
        findings
            .iter()
            .zip([(16, "offset"), (54, "offset#2"), (56, "offset#3")])
    {
        assert_eq!(finding["line"], line, "{ignored}");
        assert_eq!(finding["target"], target, "{ignored}");
    }
}

#[test]
fn scalar_switch_label_effects_preserve_literal_sql_checks() {
    let report = check("postgres-no-offset", "switch-scalar-labels-good");
    assert!(report["rules"].as_array().unwrap().is_empty(), "{report}");
}
