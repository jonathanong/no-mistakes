use super::*;

#[test]
fn case_label_executors_exclude_the_prior_bodys_fallthrough_mutations() {
    let report = check("postgres-no-offset", "switch-label-calls");
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "{report}");
    assert_eq!(findings[0]["line"], 11, "{report}");
    assert_eq!(findings[0]["target"], "offset", "{report}");
}
