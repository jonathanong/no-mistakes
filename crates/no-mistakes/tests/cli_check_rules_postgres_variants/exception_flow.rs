use super::*;

#[test]
fn incomplete_exception_paths_apply_the_dynamic_policy_in_handlers_and_finalizers() {
    let failed = check("postgres-no-offset", "exception-flow-fail");
    let findings = failed["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 4, "{failed}");
    for (finding, line) in findings.iter().zip([9, 10, 12, 21]) {
        assert_eq!(finding["target"], "unanalyzable", "{failed}");
        assert_eq!(finding["line"], line, "{failed}");
    }
    let ignored = check("postgres-no-offset", "exception-flow-ignore");
    assert!(ignored["rules"].as_array().unwrap().is_empty(), "{ignored}");
}

#[test]
fn a_safety_clause_after_a_throw_cannot_make_a_legacy_builder_complete() {
    let failed = check("postgres-bounded-statements", "exception-flow-append");
    let findings = failed["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 2, "{failed}");
    for (finding, line) in findings.iter().zip([8, 10]) {
        assert!(
            finding["target"].as_str().unwrap().contains("unanalyzable"),
            "{failed}"
        );
        assert_eq!(finding["line"], line, "{failed}");
    }
}
