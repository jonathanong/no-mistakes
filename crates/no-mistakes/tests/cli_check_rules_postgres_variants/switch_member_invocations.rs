use super::*;

#[test]
fn switch_member_invocations_follow_fail_ignore_and_executor_suppression() {
    let report =
        check_with_suppressed("postgres-no-offset", "switch-member-invocations-fail", true);
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 5, "{report}");
    for (finding, (line, target)) in findings.iter().zip([
        (5, "unanalyzable"),
        (10, "unanalyzable"),
        (15, "unanalyzable"),
        (21, "unanalyzable"),
        (26, "offset"),
    ]) {
        assert_eq!(finding["line"], line, "{report}");
        assert_eq!(finding["target"], target, "{report}");
    }
    let suppressed = report["suppressed"].as_array().unwrap();
    assert_eq!(suppressed.len(), 1, "{report}");
    assert_eq!(suppressed[0]["line"], 33, "{report}");
    assert_eq!(suppressed[0]["file"], "input.ts", "{report}");
    assert_eq!(suppressed[0]["rule"], "postgres-no-offset", "{report}");
    assert_eq!(suppressed[0]["directive"]["kind"], "nextLine", "{report}");
    assert_eq!(suppressed[0]["directive"]["line"], 32, "{report}");
    assert!(
        suppressed[0]["reason"]
            .as_str()
            .unwrap()
            .contains("OFFSET in a dynamic SELECT cannot be ruled out"),
        "{report}"
    );

    let ignored = check("postgres-no-offset", "switch-member-invocations-ignore");
    let findings = ignored["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "{ignored}");
    assert_eq!(findings[0]["line"], 26, "{ignored}");
    assert_eq!(findings[0]["target"], "offset", "{ignored}");
}

#[test]
fn switch_member_invocations_preserve_pure_calls_callbacks_and_static_sql() {
    let report = check("postgres-no-offset", "switch-member-invocations-good");
    assert!(report["rules"].as_array().unwrap().is_empty(), "{report}");
}
