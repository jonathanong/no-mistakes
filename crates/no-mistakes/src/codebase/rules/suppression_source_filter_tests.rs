// In-memory source filters and already-suppressed rules. check JSON only
// shows post-filter findings for a real rule, so these stay unit tests.
use super::*;

fn sample_finding(file: &str, line: usize) -> RuleFinding {
    RuleFinding {
        rule: "my-rule".to_string(),
        file: file.to_string(),
        line,
        message: "sample".to_string(),
        import: None,
        target: None,
    }
}

#[test]
fn suppress_rule_findings_with_source_filters_disabled_lines() {
    let source = "kept\n// no-mistakes-disable-line my-rule\n";
    let mut findings = vec![sample_finding("a.ts", 1), sample_finding("a.ts", 2)];

    suppression::suppress_rule_findings_with_source(&mut findings, source);

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].line, 1);
}

#[test]
fn already_suppressed_rules_stay_visible() {
    let temp = tempfile::tempdir().unwrap();
    let mut findings = vec![sample_finding("missing.ts", 1)];

    suppression::suppress_rule_findings_inner(temp.path(), &mut findings, None, &["my-rule"]);

    assert_eq!(findings.len(), 1);
}

#[test]
fn missing_source_file_keeps_the_finding() {
    let temp = tempfile::tempdir().unwrap();
    let mut findings = vec![sample_finding("missing.ts", 1)];

    suppression::suppress_rule_findings_inner(temp.path(), &mut findings, None, &[]);

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "missing.ts");
}
