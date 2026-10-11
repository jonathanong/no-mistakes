use super::*;

#[test]
fn unrelated_switch_paths_retain_the_two_real_sql_statements() {
    let report = check("postgres-no-offset", "unaffected-switch");
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "{report}");
    assert_eq!(findings[0]["target"], "offset", "{report}");
    assert_eq!(findings[0]["line"], 3, "{report}");
}
