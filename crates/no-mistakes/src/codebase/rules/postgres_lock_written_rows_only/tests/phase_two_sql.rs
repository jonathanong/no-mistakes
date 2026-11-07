use super::*;

#[test]
fn hashed_constants_include_bound_module_values_but_not_row_keys_or_unknown_functions() {
    let findings = check(&fixture("hashed-advisory"), &config()).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| (finding.line, finding.target.as_deref().unwrap()))
            .collect::<Vec<_>>(),
        [6, 7, 8, 9].map(|line| (line, CHECKS[2])).to_vec()
    );
}

#[test]
fn atomic_cte_writes_only_excuse_locks_on_the_same_relation_in_the_same_statement() {
    let findings = check(&fixture("atomic-cte"), &config()).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| (finding.line, finding.target.as_deref().unwrap()))
            .collect::<Vec<_>>(),
        [6, 7, 8, 9].map(|line| (line, CHECKS[3])).to_vec()
    );
}
